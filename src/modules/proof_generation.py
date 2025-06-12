"""
Module for generating proof blocks in Verus code wherever `// TODO: add proof` appears.

This module is intended to run *after* `spec_inference` if the planner detects
that proof stubs remain.  It analyzes the code and replaces every occurrence of
`// TODO: add proof` (or similar) with a proper `proof { ... }` block that
helps Verus discharge the outstanding obligations.
"""

from pathlib import Path
from typing import List

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    update_checkpoint_best,
    get_examples,
)
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
            "1. Place the `proof {}` block immediately below the comment it "
            "replaces.  Remove the `// TODO: add proof` line.\n"
            "2. Within the block, add the necessary assertions, lemma calls, or "
            "ghost variable introductions to prove the related statements.\n"
            "3. Use `assert(condition)` and, when helpful, the `by (...)` syntax "
            "(e.g., `by(nonlinear_arith)` or explicit proof steps).\n"
            "4. Do NOT modify parts of the code that are unrelated to proof.\n"
            "5. If no `// TODO: add proof` markers are present, return the code "
            "unchanged.\n"
            "6. Ensure the final code compiles under Verus and contains no "
            "remaining TODO placeholders.\n\n"
            "Return the ENTIRE file with your changes – not a diff or partial "
            "snippet."
        )

    # ---------------------------------------------------------------------
    # Helper
    # ---------------------------------------------------------------------

    def _should_skip(self, code: str) -> bool:
        """Return True if the code has no proof TODO markers."""
        return "TODO: add proof" not in code and "TODO:add proof" not in code

    # ---------------------------------------------------------------------
    # Public API – required by BaseModule
    # ---------------------------------------------------------------------

    def exec(self, context) -> str:  # type: ignore[override]
        """Run proof generation on the latest trial in *context*."""
        self.logger.info("Proof Generation ...")

        # Current code to operate on
        code = context.trials[-1].code

        # Early exit if no proof markers exist
        if self._should_skip(code):
            self.logger.info("No '// TODO: add proof' markers found – skipping proof generation.")
            return code

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
            processed_responses.append(fixed_resp if fixed_resp else resp)

        # Evaluate samples and select the best one
        output_dir = samples_dir()
        global_dir = best_dir()

        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="05_proof_generation",
            logger=self.logger,
        )

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