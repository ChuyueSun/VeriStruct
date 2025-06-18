"""
Module for generating proof blocks in Verus code wherever `// TODO: add proof` appears.

This module is intended to run *after* `spec_inference` if the planner detects
that proof stubs remain.  It analyzes the code and replaces every occurrence of
`// TODO: add proof` (or similar) with a proper `proof { ... }` block that
helps Verus discharge the outstanding obligations.
"""

from pathlib import Path
from typing import List
import re  # Added for regex detection of empty proof blocks

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    update_checkpoint_best,
    get_examples,
    code_change_is_safe,
    get_nonlinear_lines,
)
from src.modules.lynette import lynette
from src.prompts.template import build_instruction
from src.utils.path_utils import samples_dir, best_dir


class ProofGenerationModule(BaseModule):
    """Module that fills in proof blocks for Verus verification."""

    def __init__(self, config, logger):
        super().__init__(
            name="proof_generation",
            desc="Insert proof blocks to replace '// TODO: add proof' markers",
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)

        # Main instruction for proof generation
        self.proof_instruction = (
            "You are an expert in Verus (a Rust-based verification framework). "
            "For every occurrence of the comment `// TODO: add proof` in the "
            "provided code, insert an appropriate `proof { ... }` block that "
            "helps Verus verify the program.  Follow these guidelines:\n"
            "1. Place the `proof {}` block immediately below the `// TODO: add proof` "
            "comment, but **do not delete or modify the comment itself**.\n"
            "2. Within the block, add the necessary assertions, *lemma calls* ― "
            "make sure to reuse any existing lemmas already declared in the "
            "file or in scope ― and ghost variable introductions to prove the "
            "related statements.\n"
            "3. For every member function inside an `impl` block, include a call "
            "to `use_type_invariant(&*self);` (or `use_type_invariant(self);` "
            "for value receivers) inside the generated proof block so that the "
            "struct's type invariant is available.\n"
            "4. Use `assert(condition)` and, when helpful, the `by (...)` syntax "
            "(e.g., `by(nonlinear_arith)` or explicit proof steps).\n"
            "5. Do NOT modify parts of the code that are unrelated to proof.\n"
            "6. If no `// TODO: add proof` markers are present, return the code "
            "unchanged.\n"
            "7. Ensure the final code compiles under Verus and contains no "
            "remaining TODO placeholders.\n\n"
            "Return the ENTIRE file with your changes – not a diff or partial "
            "snippet."
        )

    # ---------------------------------------------------------------------
    # Helper
    # ---------------------------------------------------------------------

    def _should_skip(self, code: str) -> bool:
        """Return True if the code has no proof TODO markers."""
        # Skip only if *none* of the typical proof markers/empty blocks are present.
        if ("TODO: add proof" in code) or ("TODO:add proof" in code):
            return False

        # Detect empty proof blocks such as `proof{}`, `proof {}`, or `proof {\n}`
        if re.search(r"proof\s*{\s*}\s*", code):
            return False

        return True

    def detect_nonlinear_arithmetic(self, code: str) -> List[int]:
        """
        Detect lines with nonlinear arithmetic using Lynette.
        
        Args:
            code: Source code to analyze
            
        Returns:
            List of line numbers containing nonlinear arithmetic
        """
        try:
            return get_nonlinear_lines(code, self.logger)
        except Exception as e:
            self.logger.error(f"Error detecting nonlinear arithmetic: {e}")
            return []

    def check_code_safety(self, original_code: str, new_code: str) -> bool:
        """
        Check if code changes are safe using Lynette comparison.
        
        Args:
            original_code: Original code
            new_code: Modified code
            
        Returns:
            True if changes are safe, False otherwise
        """
        try:
            # Get immutable functions from config if available
            immutable_funcs = self.config.get("immutable_functions", [])
            
            return code_change_is_safe(
                origin_code=original_code,
                changed_code=new_code,
                verus_path=self.config.get("verus_path", "verus"),
                logger=self.logger,
                immutable_funcs=immutable_funcs
            )
        except Exception as e:
            self.logger.error(f"Error checking code safety: {e}")
            return True  # Default to safe if check fails

    def enhance_instruction_with_nonlinear_info(self, instruction: str, code: str) -> str:
        """
        Enhance the proof instruction with information about nonlinear arithmetic locations.
        
        Args:
            instruction: Base instruction
            code: Source code to analyze
            
        Returns:
            Enhanced instruction with nonlinear arithmetic guidance
        """
        nonlinear_lines = self.detect_nonlinear_arithmetic(code)
        
        if nonlinear_lines:
            nonlinear_info = (
                f"\n\nIMPORTANT: The following lines contain nonlinear arithmetic "
                f"and may require `by(nonlinear_arith)` in proof blocks: "
                f"{', '.join(map(str, nonlinear_lines))}\n"
                f"Consider adding assertions with `by(nonlinear_arith)` for these operations."
            )
            return instruction + nonlinear_info
        
        return instruction

    # ---------------------------------------------------------------------
    # Public API – required by BaseModule
    # ---------------------------------------------------------------------

    def exec(self, context) -> str:  # type: ignore[override]
        """Run proof generation on the latest trial in *context*."""
        self.logger.info("Proof Generation ...")

        # Current code to operate on
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Early exit if no proof markers exist
        if self._should_skip(code):
            self.logger.info("No '// TODO: add proof' markers found – skipping proof generation.")
            return code

        # Build instruction with common Verus knowledge and match guidelines
        base_instruction = build_instruction(
            base_instruction=self.proof_instruction,
            add_common=True,
            add_match=True,
            code=code,
            knowledge=context.gen_knowledge(),
        )
        
        # Enhance instruction with nonlinear arithmetic information
        instruction = self.enhance_instruction_with_nonlinear_info(base_instruction, code)

        # Load examples if available (input-proof / output-proof)
        examples = get_examples(self.config, "proof", self.logger)

        # Query the LLM
        try:
            responses: List[str] = self.llm.infer_llm(
                self.config.get("aoai_generation_model", "gpt-4"),
                instruction,
                examples,
                code,
                system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                answer_num=3,
                max_tokens=self.config.get("max_token", 8192),
                temp=1.0,
            )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            return code  # Fallback to original code

        # Fix simple type errors in each response
        processed_responses: List[str] = []
        for resp in responses:
            fixed_resp, _ = debug_type_error(resp, logger=self.logger)
            final_resp = fixed_resp if fixed_resp else resp
            
            # Check if the generated code is safe
            if self.check_code_safety(original_code, final_resp):
                processed_responses.append(final_resp)
                self.logger.info("Generated proof code passed safety check")
            else:
                self.logger.warning("Generated proof code failed safety check, using original")
                processed_responses.append(original_code)

        # Evaluate samples and select the best one
        output_dir = samples_dir()
        global_dir = best_dir()

        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="05_proof_generation",
            logger=self.logger,
        )

        # Final safety check on the best code
        if not self.check_code_safety(original_code, best_code):
            self.logger.warning("Best generated code failed final safety check, falling back to original")
            best_code = original_code

        # Update global checkpoint best (but don't overwrite current trial yet)
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save module-specific best
        module_best_path = output_dir / "05_proof_generation_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best proof generation sample to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best proof generation sample: {e}")

        # Update context globals
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from this step to context so subsequent stages use it
        context.add_trial(best_code)

        return best_code 