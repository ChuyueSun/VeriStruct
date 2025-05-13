"""
Module for repairing invariant errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval


class RepairInvariantModule(BaseRepairModule):
    """
    Module for repairing invariant errors.
    Handles both 'invariant not satisfied before loop' (InvFailFront) and
    'invariant not satisfied at end of loop' (InvFailEnd) errors.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_invariant",
            desc="Repair invariant failures by adding proofs or loop assertions",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the invariant repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific invariant VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair invariant error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            front_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.InvFailFront
            )
            end_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.InvFailEnd
            )
            failures = front_failures + end_failures

            if not failures:
                self.logger.warning("No invariant failures found in the last trial.")
                return code  # Return original code if no invariant error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is an invariant error
        if failure_to_fix.error not in [
            VerusErrorType.InvFailFront,
            VerusErrorType.InvFailEnd,
        ]:
            self.logger.warning(
                f"Received non-invariant error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate instruction based on error type
        if failure_to_fix.error == VerusErrorType.InvFailFront:
            return self.repair_invfail_front(context, failure_to_fix)
        else:  # VerusErrorType.InvFailEnd
            return self.repair_invfail_end(context, failure_to_fix)

    def repair_invfail_front(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair invariant not satisfied before loop error.

        Args:
            context: The current execution context
            failure_to_fix: The specific invariant VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        error_trace = failure_to_fix.trace[0]
        error_highlight = (
            error_trace.get_highlights()[0] if error_trace.get_highlights() else ""
        )

        instruction = """Your mission is to fix the invariant not satisfied error before the loop for the following code. Here are several general and possible ways to fix the error:

1. Add the assertions related to the failed loop invariant before the loop body.
2. If there are multiple loops and you believe the failed invariant is also true in preceeding loops, you should add the failed invariant to those preceeding loops as well.
3. If you believe the failed invariant is incorrect or not needed, you can modify it or delete it.

Please think twice about which way is the best to fix the error!

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "inv-front", self.logger)

        query_template = "Failed invariant before the loop\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        inv_info = line_info + error_trace.get_text() + "\n"
        query = query_template.format(inv_info, code)

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
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_invfail_front",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_invfail_end(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair invariant not satisfied at end of loop error.

        Args:
            context: The current execution context
            failure_to_fix: The specific invariant VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the invariant not satisfied error at end of the loop for the following code. Basically, you should add the assertion (in proof block) of the failed loop invariant at the end of the loop. DO NOT change the existing proof functions. If you think the failed invariant is incorrect, you can delete/correct it.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "inv-end", self.logger)

        query_template = "Failed invariant at end of the loop\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        inv_info = line_info + error_trace.get_text() + "\n"
        query = query_template.format(inv_info, code)

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
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_invfail_end",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
