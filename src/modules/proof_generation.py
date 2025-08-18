"""
Module for generating proof blocks in Verus code wherever `// TODO: add proof` appears.

This module is intended to run *after* `spec_inference` if the planner detects
that proof stubs remain.  It analyzes the code and replaces every occurrence of
`// TODO: add proof` (or similar) with a proper `proof { ... }` block that
helps Verus discharge the outstanding obligations.
"""

from pathlib import Path
from typing import List, Dict
import re  # Added for regex detection of empty proof blocks

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    update_checkpoint_best,
    get_examples,
    code_change_is_safe,
)
from src.modules.lynette import lynette
from src.prompts.template import build_instruction
from src.utils.path_utils import samples_dir, best_dir


class ProofGenerationModule(BaseModule):
    """Module that fills in proof blocks for Verus verification."""

    def __init__(self, config, logger):
        super().__init__(
            name="proof_generation",
            desc="Generate proofs for Verus functions",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for proof generation
        self.proof_instruction = (
            "You are an expert in Verus (a Rust-based verification framework). Your task is to "
            "replace every occurrence of `// TODO: add proof` or `// TODO: add invariants` with appropriate "
            "If invariants already exist, reconsider the invariants and edit them if necessary. "
            "proof blocks or loop invariants that help Verus verify the program. Follow these guidelines carefully:\n\n"
            "1. PROOF BLOCK STRUCTURE:\n"
            "   - For regular functions (`fn` or `pub fn`): Add proof blocks using the syntax `proof { ... }`\n"
            "   - For proof functions (`proof fn`): Write assertions directly in the function body - DO NOT use `proof { ... }` blocks\n"
            "   - Each proof block should be focused and minimal, containing only what's needed\n"
            "2. PROOF BLOCK CONTENTS:\n"
            "   - Start with type invariant usage (if exists): For methods in `impl` blocks, begin with:\n"
            "     * `use_type_invariant(&*self);` for reference receivers\n"
            "     * `use_type_invariant(self);` for value receivers\n"
            "   - Carefully review all existing lemmas defined in the file and invoke each one that is relevant to the current proof context, using the syntax `lemma_name(arg1, arg2, ...)`.\n"
            "     * For example, if there are lemmas about sequence bounds or modular arithmetic, call them as needed, such as `lemma_mod_auto(self.vt.len() as int)`.\n"
            "     * For lemmas about sequence properties, use the appropriate generic syntax, e.g., `broadcast use group_seq_properties`.\n"
            "     * When reasoning about sequences or specifications, ensure that all applicable modular arithmetic and sequence-related lemmas from the file are called to support your proof.\n"
            "   - Use assertions strategically with `assert(condition)`\n"
            "   - When helpful, use the `by(...)` syntax for proof steps:\n"
            "     * `by(nonlinear_arith)` for arithmetic reasoning\n"
            "     * `by { ... }` for explicit proof steps\n\n"
            "3. LOOP INVARIANTS:\n"
            "   When adding loop invariants (marked by `// TODO: add invariants`), follow these steps:\n"
            "   - Identify and add invariants for EVERY variable that is READ in the loop:\n"
            "     * For scalar variables (e.g., x, y)\n"
            "     * For array/vector elements (e.g., x[k], v[i])\n"
            "     * Include invariants about their initial values\n"
            "   - Identify and add invariants for EVERY variable that is WRITTEN in the loop:\n"
            "     * For direct assignments (e.g., y = ...)\n"
            "     * For vector/array updates (e.g., v.set(..., ...))\n"
            "     * Repeat relevant invariants even if specified earlier\n"
            "   - Fully utilize spec functions and proof functions in the invariants\n"
            "4. COMMON PROOF LOCATIONS:\n"
            "   - At function start\n"
            "   - Before loops\n"
            "   - At loop start\n"
            "   - At loop end\n"
            "   - Before key operations\n"
            "   - After key operations\n"
            "5. CONSTRAINTS:\n"
            "   - DO NOT modify any code outside of proof blocks or invariant declarations\n"
            "   - DO NOT change function signatures, types, or specifications\n"
            "   - DO NOT add new functions or types\n"
            "   - If no TODO markers exist, return code unchanged\n"
            "6. VERIFICATION:\n"
            "   - Ensure all proof blocks and invariants compile under Verus\n"
            "   - Remove all TODO placeholders\n"
            "Return the ENTIRE file with your changes – not a diff or partial snippet."
        )

    def _get_llm_responses(
        self, 
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        temperature_boost: float = 0.2,
        retry_attempt: int = 0,
        use_cache: bool = True,
    ) -> List[str]:
        """Get responses from LLM with error handling."""
        try:
            # Add retry marker to instruction to ensure cache miss
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Retry Attempt: {retry_attempt}]"
                use_cache = False  # Disable cache for retries
                
            # Log the complete query content for debugging
            self.logger.debug("=== LLM Query Content ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(f"Temperature: {1.0 + (retry_attempt * temperature_boost)}")
            self.logger.debug(f"Cache Enabled: {use_cache}")
            self.logger.debug("\n=== Instruction ===\n" + instruction)
            self.logger.debug("\n=== Code ===\n" + code)
            if examples:
                self.logger.debug("\n=== Examples ===")
                for i, ex in enumerate(examples):
                    self.logger.debug(f"\nExample {i+1} Query:\n" + ex["query"])
                    self.logger.debug(f"\nExample {i+1} Answer:\n" + ex["answer"])
            self.logger.debug("=====================")
                
            return self.llm.infer_llm(
                self.config.get("aoai_generation_model", "gpt-4"),
                instruction,
                examples or [],
                code,
                system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                answer_num=3,
                max_tokens=self.config.get("max_token", 8192),
                temp=1.0 + (retry_attempt * temperature_boost),
                use_cache=use_cache,  # Pass cache flag to LLM
            )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            return []

    def _process_responses(
        self, 
        responses: List[str], 
        original_code: str,
        context_msg: str = "",
        verus_path: str = "verus",
    ) -> List[str]:
        """Process and validate LLM responses."""
        safe_responses = []
        for response in responses:
            # Fix simple type errors
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            final_response = fixed_response if fixed_response else response

            # Check if the generated code is safe
            if code_change_is_safe(
                origin_code=original_code,
                changed_code=final_response,
                verus_path=verus_path,
                logger=self.logger,
            ):
                safe_responses.append(final_response)
                self.logger.info(f"Generated proof code passed safety check{context_msg}")
            else:
                self.logger.warning(f"Generated proof code failed safety check{context_msg}")
        return safe_responses

    # ---------------------------------------------------------------------
    # Helper
    # ---------------------------------------------------------------------

    def _should_skip(self, code: str) -> bool:
        return False
        """Return True if the code has no proof TODO markers or empty proof blocks."""
        # Skip only if *none* of the typical proof markers/empty blocks are present.
        if ("TODO: add proof" in code) or ("TODO:add proof" in code) or \
           ("TODO: add invariants" in code) or ("TODO: add invariant" in code) or \
           ("TODO: add assert" in code) or ("TODO: add asserts" in code) or \
           ("Proof body here if needed" in code):
            return False

        # Detect empty proof blocks such as `proof{}`, `proof {}`, or `proof {\n}`
        if re.search(r"proof\s*{\s*}\s*", code) or \
           re.search(r"proof\s*{\s*//[^\n]*\n\s*}\s*", code):  # Matches proof blocks with only comments
            return False

        return True

    def exec(self, context) -> str:  # type: ignore[override]
        """Run proof generation on the latest trial in *context*."""
        self.logger.info("Proof Generation ...")

        # Current code to operate on
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Early exit if no proof markers exist
        if self._should_skip(code):
            self.logger.info(
                "No '// TODO: add proof' markers found – skipping proof generation."
            )
            return code

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(f"Proof generation attempt {retry_attempt + 1}/{max_retries}")

            # Build instruction with common Verus knowledge and match guidelines
            instruction = build_instruction(
                base_instruction=self.proof_instruction,
                add_common=True,
                # add_match=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            # Load examples if available (input-proof / output-proof)
            examples = get_examples(self.config, "proof", self.logger)

            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction, 
                code, 
                examples, 
                retry_attempt=retry_attempt,
                use_cache=True,
                # use_cache=(retry_attempt == 0)    
            )
            if not responses and retry_attempt == max_retries - 1:
                return code

            safe_responses.extend(self._process_responses(
                responses, 
                original_code,
                context_msg="",
                verus_path=self.config.get("verus_path", "verus")
            ))

            if safe_responses:
                self.logger.info(f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts")
                break

            if retry_attempt < max_retries - 1:
                self.proof_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed safety checks. "
                    f"Please ensure your proof blocks do not modify any existing code "
                    f"and only add new proof blocks. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            return original_code

        # Evaluate samples and select the best one
        output_dir = samples_dir()
        global_dir = best_dir()

        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="05_proof_generation",
            logger=self.logger,
        )

        # Final safety check on the best code
        if not code_change_is_safe(
            origin_code=original_code,
            changed_code=best_code,
            verus_path=self.config.get("verus_path", "verus"),
            logger=self.logger,
        ):
            self.logger.warning(
                "Best generated code failed final safety check, falling back to original"
            )
            best_code = original_code

        # Update global checkpoint best (but don't overwrite current trial yet)
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best proof generation from this step to a module-specific file
        module_best_path = output_dir / "05_proof_generation_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best proof generation to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best proof generation: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
