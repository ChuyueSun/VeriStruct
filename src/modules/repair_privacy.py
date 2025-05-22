"""
Module for repairing privacy-related errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval


class RepairPrivacyModule(BaseRepairModule):
    """
    Module for repairing privacy-related errors.
    Handles errors like 'in requires clause of public function, cannot refer to private function'.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_privacy",
            desc="Repair privacy-related errors in requires/ensures clauses",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the privacy repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair privacy-related error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.PrivacyViolation
            )
            if not failures:
                self.logger.warning("No privacy-related failures found in the last trial.")
                return code  # Return original code if no privacy-related error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is a privacy-related error
        if failure_to_fix.error != VerusErrorType.PrivacyViolation:
            self.logger.warning(
                f"Received non-privacy error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        return self.repair_privacy_error(context, failure_to_fix)

    def repair_privacy_error(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair privacy violations in requires/ensures clauses.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the privacy violation error for the following code.
The error indicates that a public function's 'requires' or 'ensures' clause refers to a private function or field.

In Verus, when a function is public, all its preconditions (requires) and postconditions (ensures) must also be public.
This means they cannot:
1. Call private functions
2. Access private fields
3. Refer to private invariants

You need to make one of these changes:
1. Make the referred private function/field public if appropriate
2. Create a public wrapper/accessor function that encapsulates the private function/field
3. Replace the private dependency with an equivalent public specification
4. Restructure the code to avoid the privacy violation

DO NOT add `self.inv()` to pre/post-conditions if `#[verifier::type_invariant]` is used, as this creates redundancy.

Make sure to preserve the overall functionality and verification properties of the code.
Respond with the full corrected Rust code only, with no extra explanations."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "privacy", self.logger)

        query_template = "Privacy violation error:\n```\n{}```\n"
        query_template += "\nCode:\n```\n{}```\n"

        if failure_to_fix.trace:
            # Take the first trace to show the relevant snippet
            err_text = failure_to_fix.trace[0].get_text(snippet=False)
        else:
            err_text = failure_to_fix.error_text

        query = query_template.format(err_text, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_generation_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0,
        )

        # Evaluate samples and get the best one
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_privacy_error",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code 