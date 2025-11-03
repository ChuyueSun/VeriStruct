"""
Repair module for fixing requires clauses that need old(self) for &mut variables.

This module handles the error "in requires, use `old(self)` to refer to the pre-state of an &mut variable"
by replacing all instances of `self.` with `old(self).` in requires clauses. It supports:

1. Single-line requires clauses
2. Multi-line requires clauses with multiple conditions
3. Multiple self references in a single requires clause

Example:
    Input:
        pub fn push(&mut self, value: i32)
            requires self.len() < 100
        {
            // Implementation
        }

    Output:
        pub fn push(&mut self, value: i32)
            requires old(self).len() < 100
        {
            // Implementation
        }
"""

import logging
import re
from typing import Optional

from src.modules.baserepair import BaseRepairModule
from src.modules.veval import VerusError, VerusErrorType


class RepairOldSelfModule(BaseRepairModule):
    # Number of lines to search before and after the error line
    SEARCH_CONTEXT = 5

    # Patterns to identify requires clause components
    REQUIRES_KEYWORD = "requires"
    SELF_PATTERN = "self."
    OLD_SELF_PATTERN = "old(self)."

    """
    Repair module for fixing requires clauses that need old(self) for &mut variables.
    This module handles cases where a requires clause needs to use old(self) to refer
    to the pre-state of a mutable reference.

    Repair Strategy:
    1. Locate the requires clause around the error line (within 5 lines)
    2. Identify the full extent of the requires clause, handling both single-line
       and multi-line formats
    3. Replace all instances of 'self.' with 'old(self).' within the requires clause
    4. Preserve original formatting, indentation, and line breaks

    The module maintains the original code structure and only modifies the necessary
    'self.' references within the requires clause. It handles:
    - Single-line requires clauses: requires(self.len() < 100)
    - Multi-line requires clauses with multiple conditions
    - Multiple self references in a single line
    - Requires clauses with mixed self and non-self conditions
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
            self.logger.warning("No error trace available in the failure object.")
            return code

        error_trace = failure_to_fix.trace[0]
        try:
            error_line = error_trace.get_lines()[0] - 1  # Convert to 0-based index
        except (AttributeError, IndexError) as e:
            self.logger.warning(f"Failed to get error line number: {e}")
            return code

        error_text = error_trace.get_text()
        self.logger.info(f"Processing error at line {error_line + 1}: {error_text}")

        # Find the requires clause containing the error
        requires_range = self._find_requires_clause(lines, error_line)
        if requires_range is None:
            self.logger.warning(
                f"Could not find {self.REQUIRES_KEYWORD} clause near line {error_line + 1}"
            )
            return code
        requires_start, requires_end = requires_range

        # Replace self with old(self) in each line of the requires clause
        # and normalize view() usage to @ shorthand to avoid parser errors
        replacements_made = 0
        for i in range(requires_start, requires_end + 1):
            original_line = lines[i]

            # 1) Normalize any existing old(self).view() to old(self)@
            if "old(self).view()" in lines[i]:
                lines[i] = lines[i].replace("old(self).view()", "old(self)@")

            # 2) Normalize self.view() to self@ first
            if "self.view()" in lines[i]:
                lines[i] = lines[i].replace("self.view()", "self@")

            # 3) Then upgrade self@ to old(self)@
            if "self@" in lines[i]:
                lines[i] = lines[i].replace("self@", "old(self)@")

            # 4) Finally, convert remaining direct field/method access from self. to old(self).
            #    This is limited to requires clause and should not affect ensures.
            if self.SELF_PATTERN in lines[i]:
                lines[i] = lines[i].replace(self.SELF_PATTERN, self.OLD_SELF_PATTERN)

            if lines[i] != original_line:
                replacements_made += 1
                self.logger.debug(
                    f"Line {i + 1}: Rewrote requires from '{original_line}' to '{lines[i]}'"
                )

        if replacements_made == 0:
            self.logger.warning(
                f"No '{self.SELF_PATTERN}' references found in {self.REQUIRES_KEYWORD} clause to replace"
            )
        else:
            self.logger.info(
                f"Made {replacements_made} replacements of '{self.SELF_PATTERN}' with '{self.OLD_SELF_PATTERN}'"
            )

        return "\n".join(lines)

    def _find_requires_clause(
        self, lines: list[str], error_line: int
    ) -> Optional[tuple[int, int]]:
        """
        Find the requires clause containing or near the error line.

        Args:
            lines: List of code lines
            error_line: The 0-based line number where the error was found

        Returns:
            A tuple of (start_line, end_line) if found, None otherwise.
            Both line numbers are 0-based indices into the lines list.
        """
        # Define the search range around the error line
        search_start = max(0, error_line - self.SEARCH_CONTEXT)
        search_end = min(len(lines), error_line + self.SEARCH_CONTEXT)

        # State for tracking requires clause
        requires_start = None
        requires_end = None
        in_requires = False
        paren_count = 0  # For tracking nested parentheses

        # Look for requires clause in the search range
        for i in range(search_start, search_end):
            line = lines[i].rstrip()
            stripped = line.strip()

            # Skip empty lines unless we're in a requires clause
            if not stripped and not in_requires:
                continue

            # Start of requires clause
            if self.REQUIRES_KEYWORD in line and not in_requires:
                requires_start = i
                requires_end = i
                in_requires = True
                # Count opening parentheses in the rest of the line
                paren_count = line[line.index(self.REQUIRES_KEYWORD) :].count("(")
                paren_count -= line[line.index(self.REQUIRES_KEYWORD) :].count(")")
                self.logger.debug(f"Found requires clause starting at line {i + 1}")
                continue

            # Inside requires clause
            if in_requires:
                # Update parentheses count
                paren_count += stripped.count("(") - stripped.count(")")

                # Check for end conditions
                if stripped == "{":  # Function body start
                    in_requires = False
                    self.logger.debug(
                        f"Found end of requires clause at line {i + 1} (function body)"
                    )
                elif paren_count == 0 and stripped.endswith(
                    ")"
                ):  # Balanced parentheses
                    requires_end = i
                    in_requires = False
                    self.logger.debug(
                        f"Found end of requires clause at line {i + 1} (balanced parens)"
                    )
                elif stripped and not stripped.endswith(
                    ","
                ):  # Non-empty line without continuation
                    requires_end = i
                    in_requires = False
                    self.logger.debug(
                        f"Found end of requires clause at line {i + 1} (no continuation)"
                    )
                elif stripped:  # Non-empty line with potential continuation
                    requires_end = i
                    self.logger.debug(f"Found requires clause content at line {i + 1}")

        # Return None if we didn't find a requires clause
        if requires_start is None:
            return None

        # Log the found range
        self.logger.debug(
            f"Found requires clause from line {requires_start + 1} to {requires_end + 1}"
        )
        return (requires_start, requires_end)
