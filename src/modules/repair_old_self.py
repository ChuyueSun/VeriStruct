"""
Repair module for fixing requires clauses that need old(self) for &mut variables.
"""

import logging
import re
from typing import Optional

from src.modules.baserepair import BaseRepairModule
from src.modules.veval import VerusError, VerusErrorType


class RepairOldSelfModule(BaseRepairModule):
    """
    Repair module for fixing requires clauses that need old(self) for &mut variables.
    This module handles cases where a requires clause needs to use old(self) to refer
    to the pre-state of a mutable reference.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        """
        Initialize the repair module.

        Args:
            config: Configuration dictionary
            logger: Logger instance
            immutable_funcs: List of function names that should not be modified
        """
        super().__init__(
            "repair_old_self",
            "Fixes requires clauses to use old(self) for &mut variables",
            config,
            logger,
            immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the old(self) repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair old(self) error in requires clause...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.RequiresOldSelf
            )
            if not failures:
                self.logger.warning("No old(self) failures found in the last trial.")
                return code  # Return original code if no old(self) error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is an old(self) error
        if failure_to_fix.error != VerusErrorType.RequiresOldSelf:
            self.logger.warning(
                f"Received non-old(self) error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        return self.repair_old_self_error(context, failure_to_fix)

    def repair_old_self_error(self, context, failure_to_fix: VerusError) -> str:
        """
        Fix a requires clause to use old(self) for &mut variables.

        Args:
            context: The current execution context
            failure_to_fix: The specific error to fix

        Returns:
            The repaired code string
        """
        code = context.trials[-1].code
        lines = code.split("\n")

        # Get the error location from the trace
        if not failure_to_fix.trace:
            self.logger.warning("No error trace available.")
            return code

        error_trace = failure_to_fix.trace[0]
        error_line = error_trace.get_line_number() - 1  # Convert to 0-based index
        error_text = error_trace.get_text()

        # Find the requires clause containing the error
        requires_pattern = r"requires\s*\([^)]*\)"
        requires_match = None
        for i in range(max(0, error_line - 5), min(len(lines), error_line + 5)):
            if "requires" in lines[i]:
                match = re.search(requires_pattern, lines[i])
                if match:
                    requires_match = match
                    error_line = i
                    break

        if not requires_match:
            self.logger.warning("Could not find requires clause.")
            return code

        # Replace self with old(self) in the requires clause
        old_requires = requires_match.group(0)
        new_requires = old_requires.replace("self.", "old(self).")

        # Update the line with the fixed requires clause
        lines[error_line] = lines[error_line].replace(old_requires, new_requires)

        return "\n".join(lines)
