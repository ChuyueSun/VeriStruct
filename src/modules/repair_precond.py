"""
Module for repairing Precondition errors in Verus code.
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
from src.utils.path_utils import best_dir, samples_dir


class RepairPrecondModule(BaseRepairModule):
    """
    Module for repairing precondition not satisfied errors.
    It tries to fix errors by adding proof blocks.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_precond",
            desc="Repair precondition failures by adding proof blocks",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the precondition repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific precondition VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair precondition error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            precond_failures = last_trial.eval.get_failures(error_type=VerusErrorType.PreCondFail)
            veclen_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.PreCondFailVecLen
            )
            private_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.require_private
            )

            failures = precond_failures + veclen_failures + private_failures

            if not failures:
                self.logger.warning("No precondition failures found in the last trial.")
                return code  # Return original code if no precondition error
            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is actually a precondition-related error
        if failure_to_fix.error not in [
            VerusErrorType.PreCondFail,
            VerusErrorType.PreCondFailVecLen,
            VerusErrorType.require_private,
        ]:
            self.logger.warning(
                f"Received non-precondition error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate repair method based on error type
        if failure_to_fix.error == VerusErrorType.PreCondFail:
            return self.repair_precond_fail(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.PreCondFailVecLen:
            return self.repair_precond_veclen(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.require_private:
            return self.repair_require_private(context, failure_to_fix)

    def repair_precond_fail(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a precondition failure error.

        Args:
            context: The current execution context
            failure_to_fix: The specific precondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing precondition failure error...")
        code = context.trials[-1].code

        # Normal route of precondition fixing
        instruction = """Your mission is to fix the precondition not satisfied error for the following code. Basically, you should add the proof blocks related to the pre-condition check just before the invocation of the function. Note, DO NOT change the proof function whose pre-condition is not satisfied. You can use the pre-conditions of the current function, invariants of the current loop, and the pre-conditions of the called functions to fix the error.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        examples = get_examples(self.config, "precond", self.logger)
        query_template = "Failed pre-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        if len(failure_to_fix.trace) < 2:
            self.logger.error("Precondition error trace is too short to process.")
            return code

        precond_trace, location_trace = failure_to_fix.trace[0], failure_to_fix.trace[1]
        if location_trace.label == VerusErrorLabel.FailedThisPreCond:
            precond_trace, location_trace = location_trace, precond_trace

        pre_cond_info = precond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(pre_cond_info, location_info, code)

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
            samples=(responses + [code]) if responses else [code],
            output_dir=output_dir,
            prefix="repair_precond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_precond_veclen(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a precondition failure error due to vector length.

        Args:
            context: The current execution context
            failure_to_fix: The specific precondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing precondition failure error due to vector length...")
        code = context.trials[-1].code

        # Extract error information from the trace
        if len(failure_to_fix.trace) < 2:
            self.logger.error("Precondition error trace is too short to process.")
            return code

        precond_trace, location_trace = failure_to_fix.trace[0], failure_to_fix.trace[1]
        if location_trace.label == VerusErrorLabel.FailedThisPreCond:
            precond_trace, location_trace = location_trace, precond_trace

        error_line = location_trace.lines[0]
        error_code = location_trace.get_text().strip()

        instruction = f"""Your mission is to fix the vector length precondition error in the following code.

For the expression `{error_code}` on Line {error_line}, you need to:

1. Identify all array/vector accesses (e.g., vec[k], vec.set(k, ..)) in this expression
2. For EACH array/vector access, add appropriate proof blocks that establish:
   - The vector length requirement (e.g., vec.len() == expected_length)
   - Index bounds (e.g., k >= 0 && k < vec.len())
3. Use available information from:
   - Current function's preconditions
   - Active loop invariants
   - Called functions' preconditions

Important Requirements:
- DO NOT modify the function whose precondition is not satisfied
- DO NOT change function signatures or pre/post conditions
- ONLY add proof blocks before the vector access
- Add proof blocks just before the invocation point where the precondition fails
- Return the complete program with your changes integrated

Response Format:
- Return ONLY the Rust code without any explanation
- Include the entire program, not just the added proof blocks"""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        examples = get_examples(self.config, "precond", self.logger)
        query_template = "Failed pre-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        pre_cond_info = precond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(pre_cond_info, location_info, code)

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
            samples=(responses + [code]) if responses else [code],
            output_dir=output_dir,
            prefix="repair_precond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_require_private(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair a require private error.

        Args:
            context: The current execution context
            failure_to_fix: The specific precondition VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Repairing require private error...")
        code = context.trials[-1].code

        # Normal route of precondition fixing
        instruction = """Your mission is to fix the precondition not satisfied error for the following code. Basically, you should add the proof blocks related to the pre-condition check just before the invocation of the function. Note, DO NOT change the proof function whose pre-condition is not satisfied. You can use the pre-conditions of the current function, invariants of the current loop, and the pre-conditions of the called functions to fix the error.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        examples = get_examples(self.config, "precond", self.logger)
        query_template = "Failed pre-condition\n```\n{}```\n"
        query_template += "Failed location\n```\n{}```\n"
        query_template += "\nCode\n```{}```\n"

        if len(failure_to_fix.trace) < 2:
            self.logger.error("Precondition error trace is too short to process.")
            return code

        precond_trace, location_trace = failure_to_fix.trace[0], failure_to_fix.trace[1]
        if location_trace.label == VerusErrorLabel.FailedThisPreCond:
            precond_trace, location_trace = location_trace, precond_trace

        pre_cond_info = precond_trace.get_text() + "\n"
        location_info = f"Line {location_trace.lines[0]}-{location_trace.lines[1]}:\n"
        location_info += location_trace.get_text() + "\n"
        query = query_template.format(pre_cond_info, location_info, code)

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
            samples=(responses + [code]) if responses else [code],
            output_dir=output_dir,
            prefix="repair_precond",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
