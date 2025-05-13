"""
Module for repairing mode-related errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval


class RepairModeModule(BaseRepairModule):
    """
    Module for repairing mode-related errors.
    Handles errors like 'cannot call function with mode exec'.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_mode",
            desc="Repair mode-related errors like exec/proof/spec mode mismatches",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the mode repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair mode-related error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.CannotCallFunc
            )
            if not failures:
                self.logger.warning("No mode-related failures found in the last trial.")
                return code  # Return original code if no mode-related error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is a mode-related error
        if failure_to_fix.error != VerusErrorType.CannotCallFunc:
            self.logger.warning(
                f"Received non-mode error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        return self.repair_mode_error(context, failure_to_fix)

    def repair_mode_error(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair mode mismatch errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the mode error for the following code.
The error indicates that a function with a spec/proof mode is being called
from an exec context or vice versa.

Verus has 3 modes:
1. `exec` - Executable code (default for `fn`)
2. `spec` - Specification code (default for `ghost fn`)
3. `proof` - Proof code (default for `proof fn`)

You need to make one of these changes:
1. Wrap the problematic code in the appropriate mode block, e.g., `proof { ... }` or `spec { ... }`
2. Adjust the function being called to be compatible with the calling context
3. Reimplement the functionality in a way that respects mode constraints
4. Add a trusted function that can bridge between spec and exec modes

Make sure to preserve the overall functionality of the code.
Respond with the full corrected Rust code only, with no extra explanations."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "mode", self.logger)

        query_template = "Mode mismatch error:\n```\n{}```\n"
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
            prefix="repair_mode_error",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
