"""
Module for repairing Precondition errors in Verus code.
"""

from typing import List, Dict, Optional, Any
from pathlib import Path
import logging

from modules.baserepair import BaseRepairModule
from modules.veval import VEval, VerusError, VerusErrorLabel, VerusErrorType
from infer import LLM
from modules.utils import get_examples, clean_code, evaluate_samples # Import necessary utilities

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
            immutable_funcs=immutable_funcs
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
            failures = last_trial.eval.get_failures(error_type=VerusErrorType.PreCondFail)
            if not failures:
                self.logger.warning("No precondition failures found in the last trial.")
                return code # Return original code if no precondition error
            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                 self.logger.warning("Could not select a failure to fix.")
                 return code
                 
        # Ensure the selected failure is actually a precondition error
        if failure_to_fix.error != VerusErrorType.PreCondFail:
            self.logger.warning(f"Received non-precondition error: {failure_to_fix.error.name}. Skipping repair.")
            return code

        # Normal route of precondition fixing
        instruction = """Your mission is to fix the precondition not satisfied error for the following code. Basically, you should add the proof blocks related to the pre-condition check just before the invocation of the function. Note, DO NOT change the proof function whose pre-condition is not satisfied. You can use the pre-conditions of the current function, invariants of the current loop, and the pre-conditions of the called functions to fix the error.

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.proof_block_info
        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge

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

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_generation_model", "gpt-4"), # Use generation model
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
            prefix="repair_precond",
            logger=self.logger
        )
        
        # Add the best result to context
        context.add_trial(best_code)
        
        return best_code 