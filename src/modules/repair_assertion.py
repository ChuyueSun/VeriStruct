"""
Module for repairing Assertion errors in Verus code.
"""

import os
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
from src.utils.lemma_utils import insert_lemma_func, insert_proof_func
from src.utils.path_utils import best_dir, samples_dir


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
        # Get lemma path from config, or use default relative to project root
        self.lemma_path = os.path.join(
            os.path.dirname(os.path.dirname(__file__)),
            config.get(
                "lemma_path", "lemmas_for_repairs"
            ),  # Fixed: removed "src/" to avoid doubling
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the assertion repair module for production code assertions.

        Note: Test assertion failures (TestAssertFail) are handled by
        RepairTestAssertionModule, not this module.

        Args:
            context: The current execution context
            failure_to_fix: The specific assertion VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair production code assertion error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            assert_failures = last_trial.eval.get_failures(error_type=VerusErrorType.AssertFail)

            if not assert_failures:
                self.logger.warning(
                    "No production code assertion failures found in the last trial."
                )
                return code  # Return original code if no assertion error
            failure_to_fix = self.get_one_failure(assert_failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is actually a production code assertion error
        if failure_to_fix.error != VerusErrorType.AssertFail:
            self.logger.warning(
                f"Received non-production assertion error: {failure_to_fix.error.name}. "
                f"This module only handles AssertFail. Skipping repair."
            )
            return code

        # Repair the assertion failure
        candidates = self.repair_assert_fail(context, failure_to_fix)

        # Evaluate candidates and return the best one
        if isinstance(candidates, list):
            output_dir = samples_dir()
            return self.evaluate_repair_candidates(code, candidates, output_dir, "repair_assertion")
        return candidates

    def repair_assert_fail(self, context, failure_to_fix: VerusError, num=1, temp=1.0) -> List[str]:
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

        # First try special assertion fixes for common patterns
        newcode = self.repair_special_assertion_error(code, failure_to_fix, num=num, temp=temp)
        if newcode:
            return [newcode]

        # Normal route of assertion fixing
        instruction = """Your mission is to fix the assertion error for the following code. Basically, you should either introduce the necessary proof blocks before the location where the assertion fails, or, if the assertion is within a loop or after a loop, you may need to add appropriate loop invariants to ensure the assertion holds true.

Response with the Rust code only, do not include any explanation."""

        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        examples = get_examples(self.config, "assert", self.logger)
        query_template = "Failed assertion\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        assertion_info = error_trace.get_text() + "\n"

        query = query_template.format(assertion_info, code)

        # Note: Prompt will be saved by LLM.infer_llm to prompts/ directory
        # No need for separate debug file
        # When called from exec(), consider refactoring to pass context
        return self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=num,
            max_tokens=8192,
            temp=temp,
        )

    def repair_special_assertion_error(
        self, code: str, failure_to_fix: VerusError, num=1, temp=1.0
    ) -> str:
        """
        Some assertions contain certain data structure / APIs that have a routine solution.
        It is a bit ad-hoc now. Eventually, this should become a dedicated lemma-lookup function.
        """
        assertion_info = failure_to_fix.trace[0].get_text()
        newcode = ""
        did_special_fix = False

        # Check for special cases that need lemmas or reveals
        if ".filter(" in assertion_info:
            self.logger.info("Special fix: adding reveal for filter")
            instruction = """Please add `reveal(Seq::filter);' at the beginning of the function where the failed assert line is located. This will help Verus understand filter and hence prove anything related to filter."""
            query_template = "Failed assertion\n```\n{}```\n\nCode\n```\n{}```\n"
            query = query_template.format(assertion_info, code)

            responses = self.llm.infer_llm(
                engine=self.config.get("aoai_debug_model", "gpt-4"),
                instruction=instruction,
                exemplars=[],
                query=query,
                system_info=self.default_system,
                answer_num=num,
                max_tokens=8192,
                temp=temp,
            )

            if responses:
                newcode = clean_code(responses[0])
                if newcode:
                    did_special_fix = True
                    code = newcode

        # Handle filter with subrange case
        if (
            ".filter(" in assertion_info
            and ".subrange(" in code
            and not ".subrange(" in assertion_info
        ):
            self.logger.info("Special fix: adding subrange lemma for filter")
            if not "lemma_seq_subrange_all" in code:
                newcode = insert_lemma_func(code, ["seq_subrange_all"], self.lemma_path)
                if newcode:
                    did_special_fix = True
                    code = newcode

        # Handle take operations
        if ".take(" in assertion_info:
            self.logger.info("Special fix: adding take lemmas")
            if not "lemma_seq_take_ascend" in code and not "lemma_seq_take_all" in code:
                newcode = insert_lemma_func(
                    code, ["seq_take_ascend", "seq_take_all"], self.lemma_path
                )
            elif not "lemma_seq_take_all" in code:
                newcode = insert_lemma_func(code, ["seq_take_all"], self.lemma_path)
            elif not "lemma_seq_take_ascend" in code:
                newcode = insert_lemma_func(code, ["seq_take_ascend"], self.lemma_path)
            else:
                newcode = code

            if newcode:
                did_special_fix = True
                code = newcode

        # Handle subrange operations
        if ".subrange(" in assertion_info:
            self.logger.info("Special fix: adding subrange lemmas")
            if not "lemma_seq_subrange_ascend" in code and not "lemma_seq_subrange_all" in code:
                newcode = insert_lemma_func(
                    code,
                    ["seq_subrange_ascend", "seq_subrange_all"],
                    self.lemma_path,
                )
            elif not "lemma_seq_subrange_all" in code:
                newcode = insert_lemma_func(code, ["seq_subrange_all"], self.lemma_path)
            elif not "lemma_seq_subrange_ascend" in code:
                newcode = insert_lemma_func(code, ["seq_subrange_ascend"], self.lemma_path)
            else:
                newcode = code

            if newcode:
                did_special_fix = True
                code = newcode

        # Handle contains operations
        if ".contains(" in assertion_info:
            self.logger.info("Special fix: adding vector lemmas")
            newcode = insert_lemma_func(code, ["vec_push", "vec_remove"], self.lemma_path)
            if newcode:
                did_special_fix = True
                code = newcode

        if did_special_fix:
            return code
        else:
            return ""
