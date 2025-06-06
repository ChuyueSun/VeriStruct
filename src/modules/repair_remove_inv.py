"""
Module for repairing redundant inv() calls in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.utils.path_utils import samples_dir, best_dir, debug_dir


class RepairRemoveInv(BaseRepairModule):
    """
    Module for removing redundant inv() calls in pre/post-conditions.
    Handles errors related to #[verifier::type_invariant] and inv() calls.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_remove_inv",
            desc="Remove redundant self.inv() calls when type_invariant is used",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the inv removal repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to remove redundant inv() calls...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.require_private
            )
            if not failures:
                failures = last_trial.eval.get_failures(
                    error_type=VerusErrorType.ensure_private
                )
                
            if not failures:
                self.logger.warning("No inv-related failures found in the last trial.")
                return code  # Return original code if no inv-related error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Check if the error is related to privacy (which could indicate redundant inv calls)
        if failure_to_fix.error not in [VerusErrorType.require_private, VerusErrorType.ensure_private]:
            self.logger.warning(
                f"Received non-privacy error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        return self.repair_remove_inv(context, failure_to_fix)

    def repair_remove_inv(self, context, failure_to_fix: VerusError) -> str:
        """
        Remove redundant inv() calls in requires/ensures when type_invariant is used.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """DO NOT add `self.inv()` to pre/post-conditions if `#[verifier::type_invariant]` is used

Respond with the full corrected code only."""
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        # Load examples
        examples = get_examples(self.config, "inv", self.logger)

        query_template = "Error message:\n```\n{}```\n"
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
        output_dir = samples_dir()
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_remove_inv",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
