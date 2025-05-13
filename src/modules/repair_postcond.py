"""
Module for repairing Postcondition errors in Verus code.
"""

import logging
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


class RepairPostcondModule(BaseRepairModule):
    """
    Module for repairing postcondition not satisfied errors.
    It tries to fix errors by adding proof blocks or modifying loop invariants.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_postcond",
            desc="Repair postcondition failures by adding proofs or modifying invariants",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the postcondition repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific postcondition VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair postcondition error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            postcond_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.PostCondFail
            )
            split_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.SplitPostFail
            )
            private_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.ensure_private
            )

            failures = postcond_failures + split_failures + private_failures

            if not failures:
                self.logger.warning(
                    "No postcondition failures found in the last trial."
                )
                return code  # Return original code if no error
            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is actually a postcondition error
        if failure_to_fix.error not in [
            VerusErrorType.PostCondFail,
            VerusErrorType.SplitPostFail,
            VerusErrorType.ensure_private,
        ]:
            self.logger.warning(
                f"Received non-postcondition error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate repair method based on error type
        if failure_to_fix.error == VerusErrorType.PostCondFail:
            return self.repair_postcond_fail(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.SplitPostFail:
            return self.repair_split_postcond_fail(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.ensure_private:
            return self.repair_ensure_private(context, failure_to_fix)

    def repair_postcond_fail(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a postcondition failure.

        Args:
            context: The current execution context
            failure_to_fix: The specific postcondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing postcondition failure...")
        code = context.trials[-1].code

        # Normal route of postcondition fixing
        instruction = f"""Your mission is to fix the post-condition not satisfied error for the following code. There are several general ways to fix the error:
The postcondition is probably correct but missing some proof. If you are confident that the post-condition is correct, you can do the following:
1. Add the proof blocks related to the post-condition at or just before the exit point where the post-condition failure occurred.
2. Modify the existing loop invariants to make them work for the post-condition.
3. If the function ends with a loop, make sure there is a loop invariant in that loop that reflects the post-condition `{failure_to_fix.trace[0].get_highlights()[0]}'.
If you are not sure about the correctness of the post-condition, you may weaken the post-condition or remove it.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        examples = get_examples(self.config, "postcond", self.logger)
        query_template = "Failed post-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        if len(failure_to_fix.trace) < 2:
            self.logger.error("Postcondition error trace is too short to process.")
            return code

        location_trace, postcond_trace = (
            failure_to_fix.trace[0],
            failure_to_fix.trace[1],
        )
        if location_trace.label == VerusErrorLabel.FailedThisPostCond:
            location_trace, postcond_trace = postcond_trace, location_trace

        post_cond_info = f"Line {postcond_trace.lines[0]}-{postcond_trace.lines[1]}:\n"
        post_cond_info += postcond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(post_cond_info, location_info, code)

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
            prefix="repair_postcond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_split_postcond_fail(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a split postcondition failure.

        Args:
            context: The current execution context
            failure_to_fix: The specific postcondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing split postcondition failure...")
        code = context.trials[-1].code

        # Normal route of postcondition fixing
        instruction = f"""Your mission is to fix the split post-condition not satisfied error for the following code. There are several general ways to fix the error:
The postcondition is probably correct but missing some proof. If you are confident that the post-condition is correct, you can do the following:
1. Add the proof blocks related to the post-condition at or just before the exit point where the post-condition failure occurred.
2. Modify the existing loop invariants to make them work for the post-condition.
3. If the function ends with a loop, make sure there is a loop invariant in that loop that reflects the post-condition `{failure_to_fix.trace[0].get_highlights()[0]}'.
If you are not sure about the correctness of the post-condition, you may weaken the post-condition or remove it.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        examples = get_examples(self.config, "postcond", self.logger)
        query_template = "Failed post-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        if len(failure_to_fix.trace) < 2:
            self.logger.error("Postcondition error trace is too short to process.")
            return code

        location_trace, postcond_trace = (
            failure_to_fix.trace[0],
            failure_to_fix.trace[1],
        )
        if location_trace.label == VerusErrorLabel.FailedThisPostCond:
            location_trace, postcond_trace = postcond_trace, location_trace

        post_cond_info = f"Line {postcond_trace.lines[0]}-{postcond_trace.lines[1]}:\n"
        post_cond_info += postcond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(post_cond_info, location_info, code)

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
            prefix="repair_postcond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_ensure_private(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a private field access in ensures clause.

        Args:
            context: The current execution context
            failure_to_fix: The specific postcondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing private field access in ensures clause...")
        code = context.trials[-1].code

        # Normal route of postcondition fixing
        instruction = """Your mission is to fix the private field access error in the ensures clause. When a private field is accessed in an ensures clause, it causes an error because ensures clauses should only reference public state.

Common fixes include:
1. Replace private field access with a public accessor method or property
2. Create a public ghost function/method that exposes the needed information
3. Rewrite the ensures clause to use only public state
4. If appropriate, make the field public (with pub keyword)

Response with the Rust code only, do not include any explanation."""
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        examples = get_examples(self.config, "postcond", self.logger)
        query_template = "Failed post-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        if len(failure_to_fix.trace) < 2:
            self.logger.error("Postcondition error trace is too short to process.")
            return code

        location_trace, postcond_trace = (
            failure_to_fix.trace[0],
            failure_to_fix.trace[1],
        )
        if location_trace.label == VerusErrorLabel.FailedThisPostCond:
            location_trace, postcond_trace = postcond_trace, location_trace

        post_cond_info = f"Line {postcond_trace.lines[0]}-{postcond_trace.lines[1]}:\n"
        post_cond_info += postcond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(post_cond_info, location_info, code)

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
            prefix="repair_postcond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
