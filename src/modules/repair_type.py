"""
Module for repairing type-related errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import (
    clean_code,
    evaluate_samples,
    fix_one_type_error_in_code,
    get_examples,
)
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval


class RepairTypeModule(BaseRepairModule):
    """
    Module for repairing type-related errors.
    Handles mismatched types, type annotations, constructor type invariants, etc.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_type",
            desc="Repair type-related errors including mismatched types and annotations",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the type repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific type VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair type error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            type_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.MismatchedType
            )
            annotation_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.TypeAnnotation
            )
            constructor_failures = last_trial.eval.get_failures(
                error_type=VerusErrorType.ConstructorFailTypeInvariant
            )

            failures = type_failures + annotation_failures + constructor_failures
            if not failures:
                self.logger.warning("No type-related failures found in the last trial.")
                return code  # Return original code if no type error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Ensure the selected failure is a type-related error
        if failure_to_fix.error not in [
            VerusErrorType.MismatchedType,
            VerusErrorType.TypeAnnotation,
            VerusErrorType.ConstructorFailTypeInvariant,
        ]:
            self.logger.warning(
                f"Received non-type error: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

        # Choose appropriate repair function based on error type
        if failure_to_fix.error == VerusErrorType.MismatchedType:
            result = self.repair_mismatched_type(context, failure_to_fix)
            if result:
                return result
        elif failure_to_fix.error == VerusErrorType.TypeAnnotation:
            return self.repair_type_annotation(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.ConstructorFailTypeInvariant:
            return self.repair_constructor_type_invariant(context, failure_to_fix)

        # Fallback to default repair if specific repair fails
        self.logger.warning(
            f"Specific repair method for {failure_to_fix.error.name} failed. Using generic approach."
        )
        return self.default_type_repair(context, failure_to_fix)

    def repair_mismatched_type(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair mismatched type errors using automated fixes first, then LLM-based approach.

        Args:
            context: The current execution context
            failure_to_fix: The specific type VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        # First, try to fix the type error automatically
        if failure_to_fix.trace:
            newcode = fix_one_type_error_in_code(
                code, failure_to_fix.trace[0], verbose=False
            )
            if newcode and newcode != code:
                self.logger.info("Automatically fixed type error.")
                context.add_trial(newcode)
                return newcode

        # If automatic fix fails, use LLM-based approach
        instruction = """Your mission is to fix the mismatched type error in the following Verus code.
Please carefully examine the error message to identify the issue and make the necessary changes to ensure type consistency.
Common fixes include:
1. Adding appropriate type casts
2. Changing variable types to match expected types
3. Fixing type parameters for generic types
4. Ensuring return types match function signatures

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "type", self.logger)

        query_template = "Mismatched type error\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

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
            prefix="repair_mismatched_type",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_type_annotation(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair type annotation errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific type VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the type annotation error for the following code. Typically, this involves adding an explicit type parameter to `None`, for example:

    ret == None::<T>

(where `T` is the correct type inferred from the context). If you are not certain, do your best to infer the type from nearby definitions, function signatures, or variable usage.

Respond with the **fixed Rust code only** and do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "type_annotation", self.logger)

        query_template = "Type annotation needed:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

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
            prefix="repair_type_annotation",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_constructor_type_invariant(
        self, context, failure_to_fix: VerusError
    ) -> str:
        """
        Repair constructor type invariant errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific type VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the constructor so that its declared type invariant is satisfied.
Often, this means adding `requires` so that the `inv(&self) -> bool` function is true.
Respond with the **fixed Rust code only** and do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "constructor_type_invariant", self.logger)

        query_template = "In constructor, the declared type invariant is not satisfied:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

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
            prefix="repair_constructor_type_invariant",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def default_type_repair(self, context, failure_to_fix: VerusError) -> str:
        """
        Generic approach for repairing type errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific type VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the type error in the following Verus code.
Please carefully analyze the error message and make the necessary changes to resolve it.
Respond with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge

        query_template = "Type error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
            instruction=instruction,
            exemplars=[],
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
            prefix="repair_default_type",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
