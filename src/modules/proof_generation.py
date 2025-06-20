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
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for proof generation
        self.proof_instruction = (
            "You are an expert in Verus (a Rust-based verification framework). Your task is to "
            "replace every occurrence of `// TODO: add proof` with appropriate proof blocks that "
            "help Verus verify the program. Follow these guidelines carefully:\n\n"
            "1. PROOF BLOCK STRUCTURE:\n"
            "   - Add proof blocks using the syntax `proof { ... }`\n"
            "   - Each proof block should be focused and minimal, containing only what's needed\n"
            "2. PROOF BLOCK CONTENTS:\n"
            "   - Start with type invariant usage (if exists): For methods in `impl` blocks, begin with:\n"
            "     * `use_type_invariant(&*self);` for reference receivers\n"
            "     * `use_type_invariant(self);` for value receivers\n"
            "   - Add necessary lemma calls, reusing existing lemmas from the file or in scope\n"
            "   - Use assertions strategically with `assert(condition)`\n"
            "   - When helpful, use the `by(...)` syntax for proof steps:\n"
            "     * `by(nonlinear_arith)` for arithmetic reasoning\n"
            "     * `by { ... }` for explicit proof steps\n\n"
            "3. SPECIAL CASES:\n"
            "   - For modular arithmetic or index-based postconditions:\n"
            "     * Look for and use modular arithmetic lemmas (e.g., `lemma_mod_auto`)\n"
            "     * Call these lemmas before using modular operations\n"
            "4. CONSTRAINTS:\n"
            "   - DO NOT modify any code outside of proof blocks\n"
            "   - DO NOT change function signatures, types, or specifications\n"
            "   - DO NOT add new functions or types\n"
            "   - If no `// TODO: add proof` markers exist, return code unchanged\n\n"
            "5. VERIFICATION:\n"
            "   - Ensure all proof blocks compile under Verus\n"
            "   - Remove all TODO placeholders\n"
            "Return the ENTIRE file with your changes – not a diff or partial snippet."
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
        for retry_attempt in range(max_retries):
            self.logger.info(f"Proof generation attempt {retry_attempt + 1}/{max_retries}")

            # Build instruction with common Verus knowledge and match guidelines
            instruction = build_instruction(
                base_instruction=self.proof_instruction,
                add_common=True,
                add_match=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            # Load examples if available (input-proof / output-proof)
            examples = get_examples(self.config, "proof", self.logger)

            # Query the LLM with increasing temperature on retries
            try:
                responses: List[str] = self.llm.infer_llm(
                    self.config.get("aoai_generation_model", "gpt-4"),
                    instruction,
                    examples,
                    code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 8192),
                    temp=1.0 + (retry_attempt * 0.2),  # Increase temperature on retries
                )
            except Exception as e:
                self.logger.error(f"Error during LLM inference: {e}")
                if retry_attempt == max_retries - 1:
                    return code  # Fallback to original code on last attempt
                continue

            # Fix simple type errors in each response
            processed_responses: List[str] = []
            safe_responses: List[str] = []
            for resp in responses:
                fixed_resp, _ = debug_type_error(resp, logger=self.logger)
                final_resp = fixed_resp if fixed_resp else resp

                # Check if the generated code is safe
                if code_change_is_safe(
                    origin_code=original_code,
                    changed_code=final_resp,
                    verus_path=self.config.get("verus_path", "verus"),
                    logger=self.logger,
                ):
                    processed_responses.append(final_resp)
                    safe_responses.append(final_resp)
                    self.logger.info("Generated proof code passed safety check")
                else:
                    self.logger.warning(
                        "Generated proof code failed safety check, will retry"
                    )
                    processed_responses.append(original_code)

            # If we have safe responses, break out of retry loop
            if safe_responses:
                self.logger.info(f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts")
                break

            # If this is not the last attempt, modify instruction for retry
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

        # Save module-specific best
        module_best_path = output_dir / "05_proof_generation_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(
                f"Saved best proof generation sample to {module_best_path}"
            )
        except Exception as e:
            self.logger.error(f"Error saving best proof generation sample: {e}")

        # Update context globals
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from this step to context so subsequent stages use it
        context.add_trial(best_code)

        return best_code
