"""
Module for repairing decreases-related errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.utils.path_utils import best_dir, samples_dir


class RepairDecreaseModule(BaseRepairModule):
    """
    Module for repairing decreases-related errors.
    Handles both 'decreases not satisfied at end of loop' (DecFailEnd) and
    'decreases not satisfied at continue' (DecFailCont) errors.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_decrease",
            desc="Repair decreases failures in loops and recursive functions",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the decreases repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific decreases VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair decreases error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            end_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.DecFailEnd
            )
            cont_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.DecFailCont
            )
            failures = end_failures + cont_failures

            if not failures:
                self.logger.warning("No decreases failures found in the last trial.")
                return code  # Return original code if no decreases error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is a decreases error
        if failure_to_fix.error not in [
            VerusErrorType.DecFailEnd,
            VerusErrorType.DecFailCont,
        ]:
            self.logger.warning(
                f"Received non-decreases error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate method based on error type
        if failure_to_fix.error == VerusErrorType.DecFailEnd:
            return self.repair_decfail_end(context, failure_to_fix)
        else:  # VerusErrorType.DecFailCont
            return self.repair_decfail_cont(context, failure_to_fix)

    def repair_decfail_end(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair 'decreases not satisfied at end of loop' error.

        Args:
            context: The current execution context
            failure_to_fix: The specific decreases VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the 'decreases not satisfied at end of loop' error in the following Verus code.

The 'decreases' clause in a loop specifies a value that must strictly decrease with each iteration, proving that the loop terminates. When this error occurs, it means the specified decreases expression is not properly decreasing at the end of the loop body.

Common fixes include:
1. Adding proof blocks to establish that the decreases expression is indeed decreasing
2. Modifying the decreases expression to something that actually decreases each iteration
3. Adding assertions to help Verus understand why the value decreases
4. Fixing the loop logic to ensure the value actually decreases each iteration

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += (
            "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()
        )

        # Load examples
        examples = get_examples(self.config, "decreases-end", self.logger)

        query_template = "Decreases not satisfied at end of loop\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        error_info = line_info + error_trace.get_text() + "\n"
        query = query_template.format(error_info, code)

        # Use tracking wrapper for LLM calls

        if context is not None and hasattr(context, "infer_llm_with_tracking"):

            result = context.infer_llm_with_tracking(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
                stage="repair",
                module=self.name,
            )

            responses = result[0] if isinstance(result, tuple) else result

        else:

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
            prefix="repair_decfail_end",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_decfail_cont(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair 'decreases not satisfied at continue' error.

        Args:
            context: The current execution context
            failure_to_fix: The specific decreases VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the 'decreases not satisfied at continue' error in the following Verus code.

When using 'continue' in a loop with a decreases clause, Verus must verify that the decreases expression will still decrease when the loop continues. This error occurs when Verus can't prove that the decreases value will be smaller after the continue statement.

Common fixes include:
1. Adding proof blocks or assertions before the continue statement to establish that the decreases expression will decrease
2. Modifying the decreases expression to something that provably decreases at each continue point
3. Restructuring the loop to avoid the problematic continue statement
4. Ensuring any variables in the decreases expression are properly modified before the continue

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += (
            "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()
        )

        # Load examples
        examples = get_examples(self.config, "decreases-cont", self.logger)

        query_template = "Decreases not satisfied at continue\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        line_info = f"Line {error_trace.lines[0]}-{error_trace.lines[1]}:\n"
        error_info = line_info + error_trace.get_text() + "\n"
        query = query_template.format(error_info, code)

        # Use tracking wrapper for LLM calls

        if context is not None and hasattr(context, "infer_llm_with_tracking"):

            result = context.infer_llm_with_tracking(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
                stage="repair",
                module=self.name,
            )

            responses = result[0] if isinstance(result, tuple) else result

        else:

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
            prefix="repair_decfail_cont",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
