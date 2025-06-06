"""
Module for repairing Assertion errors in Verus code.
"""

from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import (  # Import necessary utilities
    clean_code,
    evaluate_samples,
    get_examples,
)
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.utils.path_utils import samples_dir, best_dir, debug_dir


class RepairAssertionModule(BaseRepairModule):
    """
    Module for repairing assertion errors.
    It tries to fix errors by adding proof blocks or adjusting pre/post conditions.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_assertion",
            desc="Repair assertion failures by adding proofs or modifying pre/post conditions",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the assertion repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific assertion VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair assertion error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            assert_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.AssertFail
            )
            split_assert_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.SplitAssertFail
            )

            failures = assert_failures + split_assert_failures

            if not failures:
                self.logger.warning("No assertion failures found in the last trial.")
                return code  # Return original code if no assertion error
            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is actually an assertion error
        if failure_to_fix.error not in [
            VerusErrorType.AssertFail,
            VerusErrorType.SplitAssertFail,
        ]:
            self.logger.warning(
                f"Received non-assertion error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate repair method based on error type
        if failure_to_fix.error == VerusErrorType.AssertFail:
            return self.repair_assert_fail(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.SplitAssertFail:
            return self.repair_split_assert_fail(context, failure_to_fix)

    def repair_assert_fail(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a regular assertion failure.

        Args:
            context: The current execution context
            failure_to_fix: The specific assertion VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing assertion failure...")
        code = context.trials[-1].code

        # Use the specified instruction
        instruction = """
Fix the assertion error in the given Rust code by introducing necessary proof blocks. Specifically:

1. For each `assert(P)`, analyze the preceding code to determine how `P` is derived.
2. If `P` depends on a function's return value, check if `P` can be established as a postcondition (`ensures P`) for that function.
3. Only introduce essential postconditions—avoid unnecessary additions and do not remove `#[trigger]`.
4. Do not change the test code.

**Response Format:**
Provide only the modified Rust code—no explanations.
"""

        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.proof_block_info
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        # Load examples using the utility function
        examples = get_examples(self.config, "assert", self.logger)

        query_template = "Failed assertion\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        assertion_info = error_trace.get_text() + "\n"
        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        assertion_info = line_info + assertion_info

        query = query_template.format(assertion_info, code)

        # Save query for debugging (optional)
        dbg_dir = debug_dir()
        debug_file = dbg_dir / "assert-query.txt"
        debug_file.write_text(query)
        self.logger.info(f"Saved assertion query to {debug_file}")

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
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
        best_code, best_score, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_assertion",
            logger=self.logger,
        )

        # Check if we made progress
        if best_score:
            self.logger.info(f"Assertion repair score: {best_score}")
            self.logger.info(
                f"Best code saved to {output_dir}/repair_assertion_sample_*.rs"
            )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_split_assert_fail(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a split assertion failure.

        Args:
            context: The current execution context
            failure_to_fix: The specific assertion VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing split assertion failure...")
        code = context.trials[-1].code

        # Normal route of assertion fixing
        instruction = """Your mission is to fix the split assertion error for the following code. This error typically occurs when an assertion must be satisfied in all code paths (branches like if/else) but fails in some branches.

To fix a split assertion error, you should:
1. Check all conditional branches to ensure the assertion is properly established in each one
2. Add necessary proof blocks in each branch where the assertion might fail
3. Ensure any variables used in the assertion maintain their expected values in all code paths
4. If needed, add additional assertions before conditional branches to establish necessary invariants

Note: If the assertion is inside an immutable function, you must not modify the function itself. Instead, consider adjusting the preconditions or postconditions of the called functions/methods to resolve the error.

**Response Format:**
Provide only the modified Rust code—no explanations."""

        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.proof_block_info
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        # Load examples
        examples = get_examples(self.config, "assert", self.logger)

        query_template = "Failed split assertion\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        assertion_info = error_trace.get_text() + "\n"
        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        assertion_info = line_info + assertion_info

        query = query_template.format(assertion_info, code)

        # Save query for debugging
        dbg_dir = debug_dir()
        debug_file = dbg_dir / "split-assert-query.txt"
        debug_file.write_text(query)
        self.logger.info(f"Saved split assertion query to {debug_file}")

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
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
        best_code, best_score, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_split_assertion",
            logger=self.logger,
        )

        # Check if we made progress
        if best_score:
            self.logger.info(f"Split assertion repair score: {best_score}")
            self.logger.info(
                f"Best code saved to {output_dir}/repair_split_assertion_sample_*.rs"
            )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
