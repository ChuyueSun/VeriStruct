"""
Module for repairing Assertion errors in Verus code.
"""

from typing import List, Dict, Optional, Any
from pathlib import Path
import logging

from modules.baserepair import BaseRepairModule
from modules.veval import VEval, VerusError, VerusErrorLabel, VerusErrorType
from infer import LLM
from modules.utils import get_examples, clean_code, evaluate_samples # Import necessary utilities

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
            immutable_funcs=immutable_funcs
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
            failures = last_trial.eval.get_failures(error_type=VerusErrorType.AssertFail)
            if not failures:
                self.logger.warning("No assertion failures found in the last trial.")
                return code # Return original code if no assertion error
            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                 self.logger.warning("Could not select a failure to fix.")
                 return code
                 
        # Ensure the selected failure is actually an assertion error
        if failure_to_fix.error != VerusErrorType.AssertFail:
            self.logger.warning(f"Received non-assertion error: {failure_to_fix.error.name}. Skipping repair.")
            return code

        # Normal route of assertion fixing
        instruction = """Your mission is to fix the assertion error for the following code. Basically, you should either introduce the necessary proof blocks before the location where the assertion fails, or, if the assertion is within a loop or after a loop, you may need to add appropriate loop invariants to ensure the assertion holds true.

Note: If the assertion is inside an immutable function, you must not modify the function itself. Instead, consider adjusting the preconditions or postconditions of the called functions/methods to resolve the error.

Response with the Rust code only, do not include any explanation."""

        instruction = self.add_seq_knowledge(code, instruction)
        instruction += "\n\n" + self.general_knowledge
        
        # Load examples using the migrated utility function
        examples = get_examples(self.config, "assert", self.logger)
        
        query_template = "Failed assertion\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0]
        assertion_info = error_trace.get_text() + "\n"
        query = query_template.format(assertion_info, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"), # Use debug model for repairs?
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0, # Use higher temperature for repairs?
        )
        
        # Evaluate samples and get the best one
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_assertion",
            logger=self.logger
        )
        
        # Add the best result to context
        context.add_trial(best_code)
        
        return best_code 