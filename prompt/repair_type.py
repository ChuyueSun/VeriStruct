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
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, debug_dir, samples_dir


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

        # Try to fix type errors automatically until no more progress can be made
        if failure_to_fix.trace:
            current_code = code
            while True:
                newcode = fix_one_type_error_in_code(
                    current_code, failure_to_fix.trace[0], verbose=False
                )
                if not newcode or newcode == current_code:
                    # No more progress can be made with automatic fixes
                    break
                self.logger.info("Automatically fixed type error.")
                context.add_trial(newcode)
                current_code = newcode

            if current_code != code:
                # Return if we made any progress
                return current_code

        # If automatic fix fails, use LLM-based approach
        # Base instruction for mismatched type repair
        base_instruction = """Your mission is to fix the mismatched type error in the following Verus code.
Please carefully examine the error message to identify the issue and make the necessary changes to ensure type consistency.
Common fixes include:
1. Adding appropriate type casts
2. Changing variable types to match expected types
3. Fixing type parameters for generic types
4. Ensuring return types match function signatures

Response with the Rust code only, do not include any explanation."""

        query_template = "Mismatched type error\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

        # Build complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=base_instruction,
            add_common=True,  # Add common Verus knowledge
            code=code,  # For Seq detection
            knowledge=self.general_knowledge,  # Add general knowledge
        )

        # Load examples
        examples = get_examples(self.config, "type", self.logger)

        # Get responses from LLM
        responses = self._get_llm_responses(
            instruction,
            query,
            examples,
            retry_attempt=0,  # First attempt
            use_cache=True,
            context=context,  # Pass context for appending knowledge
        )

        # Evaluate samples and get the best one
        output_dir = samples_dir()
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
        original_code = code  # Store original for safety checking

        # Base instruction for type annotation repair
        base_instruction = """Your mission is to fix the type annotation error for the following code. Typically, this involves adding an explicit type parameter to `None`, for example:

    ret == None::<T>

(where `T` is the correct type inferred from the context). If you are not certain, do your best to infer the type from nearby definitions, function signatures, or variable usage.

Respond with the **fixed Rust code only** and do not include any explanation."""

        query_template = "Type annotation needed:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Type annotation repair attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "type_annotation", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_type_annotation_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved type annotation repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                # use_cache=True,
                use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_type_annotation_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the type annotation error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
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
        original_code = code  # Store original for safety checking

        # Base instruction for constructor type invariant repair
        base_instruction = """Your mission is to fix the constructor so that its declared type invariant is satisfied.
Often, this means adding `requires` to the constructor function so that the `inv(&self) -> bool` function is true.

**IMPORTANT**:
- DO NOT add `ret.inv()` to the ensures clause - type invariants are automatically checked by Verus!

Respond with the **fixed Rust code only** and do not include any explanation."""

        query_template = "In constructor, the declared type invariant is not satisfied:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Constructor type invariant repair attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(
                self.config, "constructor_type_invariant", self.logger
            )

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir
                / f"repair_constructor_type_invariant_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(
                f"Saved constructor type invariant repair prompt to {prompt_path2}"
            )

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_constructor_type_invariant_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the constructor type invariant. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
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
        original_code = code  # Store original for safety checking

        # Base instruction for default type repair
        base_instruction = """Your mission is to fix the type error in the following Verus code.
Please carefully analyze the error message and make the necessary changes to resolve it.
Respond with the Rust code only, do not include any explanation."""

        query_template = "Type error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        error_trace = failure_to_fix.trace[0] if failure_to_fix.trace else None
        error_info = (
            error_trace.get_text() + "\n"
            if error_trace
            else failure_to_fix.error_text + "\n"
        )
        query = query_template.format(error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"Default type repair attempt {retry_attempt + 1}/{max_retries}"
            )

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_default_type_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved default type repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples=[],  # No examples for default repair
                retry_attempt=retry_attempt,
                use_cache=True,
                # use_cache=(retry_attempt == 0)
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_default_type_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the type error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
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
