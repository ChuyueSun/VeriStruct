"""
Module for repairing test assertion failures by strengthening production code postconditions.

CRITICAL: Test functions are IMMUTABLE and cannot be modified!
The solution is to strengthen the postconditions of production functions to satisfy test expectations.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, prompt_dir, samples_dir


class RepairTestAssertionModule(BaseRepairModule):
    """
    Module for repairing test assertion failures.

    STRATEGY: Test functions are immutable - we fix production code postconditions
    to satisfy test expectations, NOT modify the test assertions.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_test_assertion",
            desc="Repair test assertion failures by strengthening production code postconditions",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the test assertion repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific test assertion error to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing test assertion failure...")
        code = context.trials[-1].code
        original_code = code

        if not failure_to_fix:
            self.logger.warning(
                "No specific failure provided for test assertion repair."
            )
            return code

        # Extract error information
        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )

        # Try to identify which production function is being tested
        tested_function = self._identify_tested_function(code, error_trace)

        # Base instruction for test assertion repair
        base_instruction = """Your mission is to fix the test assertion failure by strengthening the postconditions of the production code.

**CRITICAL CONSTRAINT:**
- The test function is IMMUTABLE and CANNOT be modified!
- DO NOT change any assertions in the test function
- DO NOT modify the test function code in any way

**Your Task:**
1. Identify which production function is being tested (usually called just before the failing assertion)
2. Strengthen that function's `ensures` clause to guarantee what the test expects
3. Add necessary proof blocks to help Verus verify the postconditions
4. Make sure the strengthened postconditions are provable given the function's implementation

**Common Patterns:**
- Test expects return value property → Add to `ensures` clause
- Test expects state relationship → Add postcondition about `self@` vs `old(self)@`
- Test expects specific value → Add `ensures ret == expected_value` condition

**Example:**
If test has:
```rust
fn test() {
    let result = obj.dequeue();
    assert(result == None::<T>);  // ← Test assertion failing
}
```

Fix the production function's postcondition:
```rust
pub fn dequeue(&mut self) -> (ret: Option<T>)
    ensures
        // Add postcondition that guarantees when None is returned
        ret.is_none() ==> old(self)@.0.len() == 0,  // ← Strengthen postcondition
        ret.is_none() ==> ret == None::<T>,          // ← Add explicit guarantee
```

Respond with the **complete fixed Rust code** only, do not include explanations."""

        if tested_function:
            base_instruction += f"\n\n**Hint:** The failing test appears to be testing the `{tested_function}` function. Focus on strengthening its postconditions."

        query_template = "Test assertion failure:\n```\n{}```\n"
        query_template += "\nCode:\n```\n{}```\n"
        query = query_template.format(error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Test assertion repair attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Save prompt for debugging
            prompt_file = (
                prompt_dir() / f"repair_test_assertion_{len(context.trials)}.txt"
            )
            prompt_file.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved test assertion repair prompt to {prompt_file}")

            # Load examples
            examples = get_examples(self.config, "test_assertion", self.logger)

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                context=context,
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_test_assertion_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the test assertion. "
                    f"Remember: DO NOT modify the test function - it is immutable! "
                    f"Instead, strengthen the postconditions of the production functions being tested. "
                    f"Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def _identify_tested_function(self, code: str, error_trace) -> Optional[str]:
        """
        Try to identify which production function is being tested.

        Looks for function calls in the test code before the failing assertion.

        Args:
            code: The full source code
            error_trace: The error trace containing line information

        Returns:
            Name of the tested function, or None if not found
        """
        if not error_trace or not error_trace.lines:
            return None

        error_line = error_trace.lines[0]
        lines = code.split("\n")

        # Look backwards from error line to find recent function calls
        # Common pattern: let result = obj.function(); assert(result...);
        import re

        for i in range(max(0, error_line - 10), error_line):
            line = lines[i] if i < len(lines) else ""

            # Look for function calls like: obj.function_name()
            match = re.search(r"\.(\w+)\s*\(", line)
            if match:
                func_name = match.group(1)
                # Skip common methods that aren't the main function being tested
                if func_name not in ["push", "len", "new", "assert"]:
                    self.logger.info(
                        f"Identified tested function: {func_name} (from line {i})"
                    )
                    return func_name

        return None
