"""
Registry for repair modules in VerusAgent.
Maps error types to appropriate repair modules.
"""

import logging
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Type, Union

from src.modules.baserepair import BaseRepairModule
from src.modules.veval import VerusError, VerusErrorType


class RepairRegistry:
    """
    Registry for mapping error types to repair modules.
    This centralized mapping makes it easier to:
    1. Add new error types and repair modules
    2. Track which errors can be handled
    3. Select appropriate repair strategies
    """

    def __init__(
        self,
        config: Dict[str, Any],
        logger: logging.Logger,
        immutable_funcs: Optional[List[str]] = None,
    ):
        """
        Initialize the repair registry.

        Args:
            config: Configuration dictionary
            logger: Logger instance
            immutable_funcs: List of function names that should not be modified
        """
        self.config = config
        self.logger = logger
        self.immutable_funcs = immutable_funcs if immutable_funcs else []
        self.repair_modules = {}
        self.error_to_module_map = {}
        self.output_paths = {}

        # Timeout tracking for repair attempts
        self.repair_timeout_threshold = config.get(
            "repair_timeout", 120
        )  # 2 minutes default
        self.llm_timeout_threshold = config.get(
            "repair_llm_timeout", 60
        )  # 1 minute for LLM calls
        self.slow_repair_threshold = config.get(
            "slow_repair_threshold", 30
        )  # 30 seconds is "slow"
        self.max_repair_retries = config.get(
            "max_repair_retries", 1
        )  # Retry once on timeout
        self.error_type_timeouts = {}  # Track which error types consistently timeout

    @classmethod
    def create(
        cls,
        config: Dict[str, Any],
        logger: logging.Logger,
        immutable_funcs: Optional[List[str]] = None,
    ):
        """
        Factory method to create and initialize a registry with all available repair modules.

        Args:
            config: Configuration dictionary
            logger: Logger instance
            immutable_funcs: List of function names that should not be modified

        Returns:
            Fully initialized RepairRegistry with all repair modules registered
        """
        # Import here to avoid circular imports
        from src.modules.repair_arithmetic import RepairArithmeticModule
        from src.modules.repair_assertion import RepairAssertionModule
        from src.modules.repair_decrease import RepairDecreaseModule
        from src.modules.repair_invariant import RepairInvariantModule
        from src.modules.repair_missing import RepairMissingModule
        from src.modules.repair_mode import RepairModeModule
        from src.modules.repair_old_self import RepairOldSelfModule
        from src.modules.repair_postcond import RepairPostcondModule
        from src.modules.repair_precond import RepairPrecondModule
        from src.modules.repair_remove_inv import RepairRemoveInv
        from src.modules.repair_syntax import RepairSyntaxModule
        from src.modules.repair_test_assertion import RepairTestAssertionModule
        from src.modules.repair_type import RepairTypeModule

        # Create registry instance
        registry = cls(config, logger, immutable_funcs)

        # Initialize and register syntax repair module (general purpose)
        # This module handles both general syntax errors and Seq-specific syntax errors
        syntax_repair = RepairSyntaxModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_syntax",
            syntax_repair,
            [VerusErrorType.Other],
            "03_repair_syntax.rs",
        )

        # Initialize and register assertion repair module (for production code assertions)
        assertion_repair = RepairAssertionModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_assertion",
            assertion_repair,
            [VerusErrorType.AssertFail],  # Only production code assertions
            "04_repair_assertion.rs",
        )

        # Initialize and register test assertion repair module (for test function assertions)
        # Test functions are IMMUTABLE - this module fixes production code postconditions instead
        test_assertion_repair = RepairTestAssertionModule(
            config, logger, immutable_funcs
        )
        registry.register_module(
            "repair_test_assertion",
            test_assertion_repair,
            [VerusErrorType.TestAssertFail],  # Only test assertions
            "04_repair_test_assertion.rs",
        )

        # Initialize and register precondition repair module
        precond_repair = RepairPrecondModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_precond",
            precond_repair,
            [
                VerusErrorType.PreCondFail,
                VerusErrorType.PreCondFailVecLen,
            ],
            "05_repair_precond.rs",
        )

        # Initialize and register inv removal module
        remove_inv_repair = RepairRemoveInv(config, logger, immutable_funcs)
        registry.register_module(
            "repair_remove_inv",
            remove_inv_repair,
            [VerusErrorType.require_private, VerusErrorType.ensure_private],
            "13_repair_remove_inv.rs",
        )

        # Initialize and register postcondition repair module
        postcond_repair = RepairPostcondModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_postcond",
            postcond_repair,
            [
                VerusErrorType.PostCondFail,
                VerusErrorType.ensure_private,
            ],
            "06_repair_postcond.rs",
        )

        # Initialize and register invariant repair module
        invariant_repair = RepairInvariantModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_invariant",
            invariant_repair,
            [VerusErrorType.InvFailFront, VerusErrorType.InvFailEnd],
            "07_repair_invariant.rs",
        )

        # Initialize and register arithmetic repair module
        arithmetic_repair = RepairArithmeticModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_arithmetic",
            arithmetic_repair,
            [VerusErrorType.ArithmeticFlow],
            "08_repair_arithmetic.rs",
        )

        # Initialize and register type repair module
        type_repair = RepairTypeModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_type",
            type_repair,
            [
                VerusErrorType.MismatchedType,
                VerusErrorType.TypeAnnotation,
                VerusErrorType.ConstructorFailTypeInvariant,
            ],
            "09_repair_type.rs",
        )

        # Initialize and register decrease repair module
        decrease_repair = RepairDecreaseModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_decrease",
            decrease_repair,
            [VerusErrorType.DecFailEnd, VerusErrorType.DecFailCont],
            "10_repair_decrease.rs",
        )

        # Initialize and register missing element repair module
        missing_repair = RepairMissingModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_missing",
            missing_repair,
            [VerusErrorType.MissingImport, VerusErrorType.MissImpl],
            "11_repair_missing.rs",
        )

        # Initialize and register mode repair module
        mode_repair = RepairModeModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_mode",
            mode_repair,
            [VerusErrorType.CannotCallFunc, VerusErrorType.PubSpecVisibility],
            "12_repair_mode.rs",
        )

        # Initialize and register old(self) repair module
        old_self_repair = RepairOldSelfModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_old_self",
            old_self_repair,
            [VerusErrorType.RequiresOldSelf],
            "14_repair_old_self.rs",
        )

        # TODO: Add more specialized repair modules for other error types:
        # - RecommendNotMet

        return registry

    def register_with_context(self, context):
        """
        Register all repair modules with the given context.

        Args:
            context: The execution context
        """
        for name, module in self.repair_modules.items():
            context.register_module(name, module)

        self.logger.info(
            f"Registered repair modules: {list(self.repair_modules.keys())}"
        )

    def register_module(
        self,
        name: str,
        module: BaseRepairModule,
        error_types: List[VerusErrorType],
        output_path: str = None,
    ):
        """
        Register a repair module to handle specific error types.

        Args:
            name: Name of the repair module
            module: The repair module instance
            error_types: List of error types this module can handle
            output_path: Optional output path template for saving repair results
        """
        self.repair_modules[name] = module
        for error_type in error_types:
            self.error_to_module_map[error_type] = module
            if output_path:
                self.output_paths[error_type] = output_path

    def get_module_for_error(self, error: VerusError) -> Optional[BaseRepairModule]:
        """
        Get the appropriate repair module for a given error.

        Args:
            error: The Verus error to repair

        Returns:
            The repair module instance, or None if no module is registered
        """
        if error.error in self.error_to_module_map:
            return self.error_to_module_map[error.error]
        return None

    def get_output_path(self, error: VerusError) -> Optional[str]:
        """
        Get the output path for a given error type.

        Args:
            error: The Verus error

        Returns:
            The output path template, or None if not specified
        """
        if error.error in self.output_paths:
            return self.output_paths[error.error]
        return None

    def prioritize_failures(self, failures: List[VerusError]) -> List[VerusError]:
        """
        Prioritize failures based on error type, returning a sorted list.
        Errors are sorted according to a predefined priority order.

        Args:
            failures: List of Verus errors to prioritize

        Returns:
            Sorted list of errors with highest priority first
        """
        if not failures:
            return []

        # Define a priority order for error types based on refinement.py's get_one_failure method
        # Lower number = higher priority
        priority_order = {
            VerusErrorType.MismatchedType: 1,  # Fix type errors first
            VerusErrorType.PreCondFailVecLen: 2,  # Fix vector length errors next
            VerusErrorType.ArithmeticFlow: 3,  # Fix arithmetic overflow/underflow
            VerusErrorType.InvFailFront: 4,  # Fix invariants not satisfied before loop
            VerusErrorType.InvFailEnd: 5,  # Fix invariants not satisfied at end of loop
            VerusErrorType.ConstructorFailTypeInvariant: 6,  # Fix constructor type invariant errors
            VerusErrorType.TypeAnnotation: 7,  # Fix type annotation errors
            VerusErrorType.DecFailEnd: 8,  # Fix decreases not satisfied at end of loop
            VerusErrorType.DecFailCont: 9,  # Fix decreases not satisfied at continue
            VerusErrorType.MissingImport: 10,  # Fix missing imports
            VerusErrorType.MissImpl: 11,  # Fix missing implementations
            VerusErrorType.CannotCallFunc: 12,  # Fix mode errors
            VerusErrorType.AssertFail: 13,  # Fix assertion failures
            VerusErrorType.TestAssertFail: 14,  # Fix test assertion failures
            VerusErrorType.PreCondFail: 15,  # Fix precondition failures
            VerusErrorType.RequiresOldSelf: 16,  # Fix old(self) in requires clauses
            VerusErrorType.PostCondFail: 17,  # Fix postcondition failures
            VerusErrorType.ensure_private: 18,  # Fix private field access in ensures
            VerusErrorType.require_private: 19,  # Fix private function access in requires
            VerusErrorType.RecommendNotMet: 20,  # Fix recommendation not met errors
            # Add more error types with their priorities here
        }

        # Default priority for errors not explicitly listed
        default_priority = 100

        # Sort failures based on priority
        return sorted(
            failures, key=lambda f: priority_order.get(f.error, default_priority)
        )

    def repair_error(
        self, context, error: VerusError, output_dir: Optional[Path] = None
    ) -> Optional[str]:
        """
        Attempt to repair a specific error using the appropriate module.

        Args:
            context: The execution context
            error: The error to repair
            output_dir: Optional directory to save the repair result

        Returns:
            The repaired code if successful, None otherwise
        """
        module = self.get_module_for_error(error)
        if not module:
            self.logger.warning(
                f"No repair module registered for error type: {error.error.name}"
            )
            return None

        self.logger.info(f"Attempting {error.error.name} repair with {module.name}...")
        result = module.exec(context, error)

        if output_dir and result:
            output_path = self.get_output_path(error)
            if output_path:
                # Get file ID from environment (set in main.py)
                file_id = os.environ.get("VERUS_FILE_ID", "")
                if file_id:
                    # Insert file ID before file extension
                    base, ext = os.path.splitext(output_path)
                    output_path = f"{base}_{file_id}{ext}"

                output_file = output_dir / output_path
                output_file.write_text(result)
                self.logger.info(
                    f"Saved {error.error.name} repair result to {output_file}"
                )

        return result

    def repair_all(
        self,
        context,
        failures: List[VerusError],
        output_dir: Optional[Path] = None,
        progress_logger=None,
        round_timeout: Optional[float] = None,
        round_start_time: Optional[float] = None,
    ) -> Dict[VerusErrorType, str]:
        """
        Attempt to repair all errors in the list using appropriate modules.

        Args:
            context: The execution context
            failures: List of errors to repair
            output_dir: Optional directory to save repair results
            progress_logger: Optional progress logger to track repair operations
            round_timeout: Maximum time allowed for the entire repair round (seconds)
            round_start_time: Start time of the repair round

        Returns:
            Dictionary mapping error types to repaired code
        """
        result_map = {}

        # Helper function to check if round has timed out
        def check_round_timeout():
            if round_timeout and round_start_time:
                elapsed = time.time() - round_start_time
                if elapsed > round_timeout:
                    self.logger.warning(
                        f"⏱️ Repair round timeout reached: {elapsed:.2f}s / {round_timeout:.2f}s"
                    )
                    return True
            return False

        # Track if we've made any progress (even if we can't repair all errors)
        made_progress = False

        # If there's a compilation error, we *first* check whether the reported
        # failure list already contains a recognizable, more specific Verus
        # error (for example `ensure_private`).  If so, we delegate directly to
        # the specialised repair module for that error instead of firing the
        # generic syntax-repair.  We fall back to `repair_syntax` only when no
        # specific handler exists.

        if context.trials[-1].eval.compilation_error:
            # Determine if at least one failure maps to a non-syntax repair module
            specialised_available = False
            for f in failures:
                mod = self.get_module_for_error(f)
                if mod and mod.name != "repair_syntax":
                    specialised_available = True
                    break

            if not specialised_available:
                self.logger.info(
                    "Compilation error with no specialised handler – attempting regex-based fixes first…"
                )

                # FIRST: Try regex-based syntax fixes (fast, deterministic)
                from src.modules.repair_regex import fix_common_syntax_errors

                current_code = context.trials[-1].code
                fixed_code, was_changed = fix_common_syntax_errors(
                    current_code, self.logger
                )

                if was_changed:
                    self.logger.info(
                        "Regex-based syntax fixer made changes. Verifying..."
                    )

                    # Verify if the regex fix resolved the compilation error
                    from src.modules.veval import VEval

                    veval = VEval(fixed_code, self.logger)
                    regex_score = veval.eval_and_get_score()
                    before_score = context.trials[-1].eval.get_score()

                    if regex_score > before_score:
                        self.logger.info(
                            "✅ Regex-based fixes resolved the compilation error!"
                        )
                        context.add_trial(fixed_code)

                        if progress_logger:
                            progress_logger.add_repair(
                                "CompilationError",
                                "repair_regex",
                                before_score,
                                regex_score,
                                0.01,  # Regex fixes are nearly instant
                            )

                        # Check if all errors are fixed
                        if not context.trials[-1].eval.compilation_error:
                            failures = context.trials[-1].eval.get_failures()
                            if not failures:
                                self.logger.info(
                                    "All errors fixed by regex-based fixer!"
                                )
                                result_map["compilation"] = fixed_code
                                return result_map

                        # Continue with remaining errors if any
                        made_progress = True
                    else:
                        self.logger.info(
                            "Regex fixes didn't resolve the compilation error. Trying LLM-based repair..."
                        )
                else:
                    self.logger.info(
                        "No regex-based fixes applicable. Trying LLM-based repair..."
                    )

                # SECOND: If regex didn't fix it, try LLM-based syntax repair
                # Check timeout before attempting LLM-based repair
                if check_round_timeout():
                    self.logger.error(
                        "🚨 Repair round timed out before LLM-based syntax repair"
                    )
                    return result_map

                self.logger.info("Attempting LLM-based syntax repair…")

                # Store the state before repair
                before_score = context.trials[-1].eval.get_score()
                repair_start_time = time.time()

                compilation_result = self.repair_compilation_error(context, output_dir)

                # Calculate repair time
                repair_time = time.time() - repair_start_time

                if compilation_result:
                    from src.modules.veval import VEval

                    veval = VEval(compilation_result, self.logger)
                    after_score = veval.eval_and_get_score()

                    # Only accept the repair if it's an improvement
                    if after_score > before_score:
                        self.logger.info(
                            f"Compilation error repair was successful in {repair_time:.2f}s."
                        )
                        made_progress = True

                        context.add_trial(compilation_result)
                        last_trial = context.trials[-1]

                        # Update checkpoint best if this compilation repair is better
                        current_best_score = context.get_best_score()
                        if (
                            current_best_score is None
                            or after_score > current_best_score
                        ):
                            self.logger.info(
                                f"Updating checkpoint best after compilation error repair: {after_score}"
                            )
                            context.set_best_score(after_score)
                            context.set_best_code(compilation_result)

                        if progress_logger:
                            progress_logger.add_repair(
                                "CompilationError",
                                "repair_syntax",
                                before_score,
                                after_score,
                                repair_time,
                            )

                        # Refresh failures list after a successful compile fix
                        if not last_trial.eval.compilation_error:
                            failures = last_trial.eval.get_failures()
                            if not failures:
                                self.logger.info(
                                    "All errors fixed after compilation repair."
                                )
                                result_map["compilation"] = compilation_result
                                return result_map
                    else:
                        self.logger.warning(
                            "Syntax repair did not improve score – skipping."
                        )
                        if progress_logger:
                            progress_logger.add_repair(
                                "CompilationError",
                                "repair_syntax",
                                before_score,
                                after_score,
                                repair_time,
                            )
            else:
                self.logger.info(
                    "Compilation error appears alongside specific Verus failures – deferring to specialised repair modules."
                )

        # Check timeout after compilation error handling
        if check_round_timeout():
            self.logger.error(
                "🚨 Repair round timed out during compilation error handling"
            )
            return result_map

        # Prioritize failures
        prioritized_failures = self.prioritize_failures(failures)

        # Group failures by error type while maintaining priority order
        error_type_map = {}
        for failure in prioritized_failures:
            if failure.error not in error_type_map:
                error_type_map[failure.error] = []
            error_type_map[failure.error].append(failure)

        # Process each error type in priority order
        for error_type, type_failures in error_type_map.items():
            # Check timeout before processing each error type
            if check_round_timeout():
                self.logger.error(
                    f"🚨 Repair round timed out before processing {error_type.name}"
                )
                break

            if error_type in self.error_to_module_map:
                module = self.error_to_module_map[error_type]
                self.logger.info(
                    f"Attempting {error_type.name} repair with {module.name}..."
                )

                # Store the state before repair
                before_score = (
                    context.trials[-1].eval.get_score() if context.trials else None
                )
                repair_start_time = time.time()

                # Use the first failure of this type with timeout protection
                # Try up to (1 + max_repair_retries) times: initial attempt + retries
                max_attempts = 1 + self.max_repair_retries
                result = None
                repair_time = 0
                attempt_start = time.time()  # Initialize for exception handling

                for attempt in range(max_attempts):
                    try:
                        if attempt > 0:
                            self.logger.info(
                                f"🔄 Retrying {error_type.name} repair (attempt {attempt + 1}/{max_attempts}) "
                                f"with reduced timeout..."
                            )
                            # For retry, use shorter timeout (half of original)
                            retry_timeout = self.repair_timeout_threshold // 2

                        attempt_start = time.time()
                        result = module.exec(context, type_failures[0])
                        repair_time = time.time() - attempt_start

                        # Check if this attempt timed out
                        current_threshold = (
                            self.repair_timeout_threshold
                            if attempt == 0
                            else retry_timeout
                        )
                        if repair_time > current_threshold:
                            self.logger.warning(
                                f"⏱️ {error_type.name} repair attempt {attempt + 1} took {repair_time:.2f}s "
                                f"(threshold: {current_threshold}s)"
                            )

                            # If this was the last attempt, log error and track timeout
                            if attempt == max_attempts - 1:
                                self.logger.error(
                                    f"🚨 {error_type.name} repair EXCEEDED TIMEOUT after {max_attempts} attempts"
                                )
                                # Track timeout for this error type
                                self.error_type_timeouts[error_type.name] = (
                                    self.error_type_timeouts.get(error_type.name, 0) + 1
                                )
                                result = None  # Mark as failed
                                break
                            else:
                                # Retry with simpler approach
                                self.logger.info(
                                    f"🔄 Attempt {attempt + 1} timed out, will retry with faster settings"
                                )
                                continue

                        # Success - repair completed within timeout
                        if attempt > 0:
                            self.logger.info(
                                f"✅ {error_type.name} repair succeeded on retry (attempt {attempt + 1})"
                            )
                        elif repair_time > self.slow_repair_threshold:
                            self.logger.warning(
                                f"⏱️ {error_type.name} repair was slow: {repair_time:.2f}s "
                                f"(threshold: {self.slow_repair_threshold}s)"
                            )

                        # Break on success
                        break

                    except Exception as e:
                        attempt_time = time.time() - attempt_start
                        self.logger.error(
                            f"❌ {error_type.name} repair attempt {attempt + 1} failed with exception "
                            f"after {attempt_time:.2f}s: {e}"
                        )

                        # If last attempt, give up
                        if attempt == max_attempts - 1:
                            result = None
                            break
                        else:
                            self.logger.info(f"🔄 Will retry after exception...")
                            continue

                # If all attempts failed/timed out, skip this error type
                if result is None:
                    self.logger.warning(
                        f"⏭️ Skipping {error_type.name} - all {max_attempts} attempts failed/timed out"
                    )
                    continue

                # Validate completeness before accepting repair
                original_code = context.trials[-1].code if context.trials else ""
                if not self._check_file_completeness(result, original_code):
                    self.logger.error(
                        f"{error_type.name} repair returned incomplete/truncated code - REJECTING!"
                    )
                    self.logger.error(
                        f"Original: {len(original_code.splitlines())} lines, "
                        f"Repaired: {len(result.splitlines())} lines"
                    )
                    # Skip this repair - don't add truncated code to trials
                    continue

                # Add the repaired code as a new trial
                context.add_trial(result)

                # Note: repair_time already calculated in try block above

                # Get the trial that was added by the repair module
                if context.trials and len(context.trials) > 0:
                    after_score = context.trials[-1].eval.get_score()

                    # Check if the repair improved the score
                    if after_score <= before_score:
                        self.logger.warning(
                            f"{error_type.name} repair did not improve the score or made it worse."
                        )
                    # If repair made things worse, try fallback
                    if (
                        context.trials[-1].eval.compilation_error
                        and not before_score.compilation_error
                    ):
                        self.logger.info(
                            "Repair introduced compilation errors. Attempting fallback..."
                        )

                        # Remove the failed trial
                        context.trials.pop()

                        # Try fallback repair
                        fallback_result, fallback_score = self._try_fallback_repair(
                            context=context,
                            output_dir=output_dir,
                            max_attempts=3,
                            preserve_trial=False,
                        )

                        if fallback_result and fallback_score:
                            self.logger.info(
                                "Fallback repair improved score. Adding to trials."
                            )
                            # Add successful fallback as new trial
                            context.add_trial(fallback_result)
                            result_map[error_type] = fallback_result
                            made_progress = True

                            # Update checkpoint best if fallback is better
                            current_best_score = context.get_best_score()
                            if (
                                current_best_score is None
                                or fallback_score > current_best_score
                            ):
                                self.logger.info(
                                    f"Updating checkpoint best after fallback repair: {fallback_score}"
                                )
                                context.set_best_score(fallback_score)
                                context.set_best_code(fallback_result)
                        else:
                            self.logger.warning(
                                "Fallback repair failed. Continuing with original code."
                            )
                    else:
                        # Only add to result_map if repair was successful
                        result_map[error_type] = result
                        made_progress = True

                        # Update checkpoint best if this repair improved the score
                        if after_score > before_score:
                            current_best_score = context.get_best_score()
                            current_best_code = context.get_best_code()

                            # Update if this is better than current checkpoint best
                            if (
                                current_best_score is None
                                or after_score > current_best_score
                            ):
                                self.logger.info(
                                    f"Updating checkpoint best after {error_type.name} repair: {after_score}"
                                )
                                context.set_best_score(after_score)
                                context.set_best_code(result)

                        # Save the result if an output directory is provided
                        if output_dir and result:
                            output_path = self.get_output_path(type_failures[0])
                            if output_path:
                                self._save_repair_result(
                                    output_dir=output_dir,
                                    output_path=output_path,
                                    result=result,
                                    repair_type=error_type.name,
                                    repair_time=repair_time,
                                )

                # Log the repair in the progress logger
                if progress_logger and context.trials:
                    after_score = context.trials[-1].eval.get_score()
                    progress_logger.add_repair(
                        error_type.name,
                        module.name,
                        before_score,
                        after_score,
                        repair_time,
                    )

                # Check timeout after completing this repair
                if check_round_timeout():
                    self.logger.warning(
                        f"⏱️ Repair round timed out after completing {error_type.name} repair"
                    )
                    break
            else:
                self.logger.warning(
                    f"No repair module registered for error type: {error_type.name}"
                )

        # Log timeout statistics at the end
        if self.error_type_timeouts:
            self.logger.warning(
                f"⏱️ Timeout summary: {len(self.error_type_timeouts)} error type(s) experienced timeouts"
            )
            for err_type, count in self.error_type_timeouts.items():
                self.logger.warning(f"  - {err_type}: {count} timeout(s)")

        # If we made progress on at least some errors, return the results
        # even if we couldn't repair all errors
        return result_map

    def _check_file_completeness(self, result, original_code: str = None) -> bool:
        """
        Validate repair result completeness by checking:
        1. Brace closure (all blocks properly closed)
        2. Essential structures present (verus! block)
        3. Reasonable file length (not severely truncated)

        Args:
            result: The code to validate
            original_code: Optional original code for length comparison

        Returns:
            True if code appears complete, False if truncated or malformed
        """
        if isinstance(result, list):
            lines = result
            result_str = "\n".join(result)
        else:
            lines = result.splitlines()
            result_str = result

        self.logger.info("Starting file completeness check...")

        # Check 1: Minimum viable length (should have at least 20 lines for any Verus code)
        if len(lines) < 20:
            self.logger.warning(
                f"File too short to be complete: {len(lines)} lines (minimum 20 expected)"
            )
            return False

        # Check 2: Length comparison with original (if provided)
        if original_code is not None:
            original_lines = original_code.splitlines()
            length_ratio = (
                len(lines) / len(original_lines) if len(original_lines) > 0 else 0
            )

            # File shouldn't shrink by more than 30% (allows some comment/whitespace removal)
            if length_ratio < 0.7:
                self.logger.warning(
                    f"File appears severely truncated: {len(lines)} lines vs {len(original_lines)} original "
                    f"({length_ratio:.1%} of original)"
                )
                return False

            # File shouldn't grow by more than 50% (suspicious expansion)
            if length_ratio > 1.5:
                self.logger.warning(
                    f"File suspiciously expanded: {len(lines)} lines vs {len(original_lines)} original "
                    f"({length_ratio:.1%} of original)"
                )
                return False

        # Check 3: Essential Verus structures present
        has_verus_block = "verus!" in result_str
        has_use_vstd = "use vstd::" in result_str or "use vstd:" in result_str

        if not has_verus_block:
            self.logger.warning("Missing 'verus!' block - file may be truncated")
            return False

        # Track braces
        open_braces = 0
        brace_positions = []  # Track line numbers of brace changes

        for i, line in enumerate(lines):
            stripped = line.strip()
            if not stripped:
                continue

            # Track braces
            open_count = stripped.count("{")
            close_count = stripped.count("}")

            if open_count > 0 or close_count > 0:
                brace_positions.append((i + 1, open_count, close_count))

            open_braces += open_count
            open_braces -= close_count

            if open_braces < 0:
                self.logger.warning(f"Extra closing braces at line {i + 1}")
                return False

        # Validate brace closure
        if open_braces != 0:
            self.logger.warning(
                f"Unclosed blocks detected: {open_braces} unclosed braces"
            )
            if open_braces > 0:
                self.logger.warning("Some blocks were not closed")
            else:
                self.logger.warning("Extra closing braces found")
            return False

        self.logger.info(
            f"✓ File structure validation passed: {len(lines)} lines, "
            f"all blocks properly closed, essential structures present"
        )
        return True

    def _check_file_size(
        self, result: Union[str, List[str]], original_size: Optional[int] = None
    ) -> bool:
        """
        Validate repair result size and completeness.

        Args:
            result: The repair result (either a string or list of strings)
            original_size: Optional size of original file for comparison

        Returns:
            bool: True if size and structure seem valid, False otherwise
        """
        # Basic size check - files shouldn't be tiny
        min_size = 100  # Minimum reasonable size for a Verus file

        # Convert list to string if needed
        if isinstance(result, list):
            result = "\n".join(result)

        # Get size in bytes and lines
        result_bytes = len(result.encode("utf-8"))
        result_lines = len(result.splitlines())

        # Log sizes for debugging
        self.logger.info(
            f"Repair result size: {result_bytes} bytes, {result_lines} lines"
        )

        if result_bytes < min_size:
            self.logger.warning(
                f"Repair result suspiciously small: {result_bytes} bytes"
            )
            return False

        # If we have original size, compare
        if original_size:
            # Allow some variance but catch major discrepancies
            size_ratio = result_bytes / original_size
            if size_ratio < 0.5:  # Less than 50% of original
                self.logger.warning(
                    f"Repair result much smaller than original: {size_ratio:.2%}"
                )
                return False

        # Check structural completeness (no original_code available here)
        return self._check_file_completeness(result, original_code=None)

    def _save_repair_result(
        self,
        output_dir: Path,
        output_path: str,
        result: Union[str, List[str]],
        repair_type: str,
        repair_time: Optional[float] = None,
    ) -> None:
        """
        Helper method to save repair results to a file.

        Args:
            output_dir: Directory to save the result
            output_path: Base path for the output file
            result: The repair result to save (string or list of strings)
            repair_type: Type of repair (for logging)
            repair_time: Optional repair time in seconds
        """
        # Convert list to string if needed
        if isinstance(result, list):
            result = "\n".join(result)

        # Validate file completeness and size (no original_code available in this context)
        if not self._check_file_completeness(result, original_code=None):
            self.logger.error(
                f"Skipping save of structurally incomplete repair result for {repair_type}"
            )
            return

        # Note: _check_file_size also calls _check_file_completeness internally,
        # but we check here first for early rejection and clearer error messages
        if not self._check_file_size(result):
            self.logger.warning(
                f"Skipping save of invalid size repair result for {repair_type}"
            )
            return

        # Get file ID from environment
        file_id = os.environ.get("VERUS_FILE_ID", "")
        if file_id:
            # Insert file ID before file extension
            base, ext = os.path.splitext(output_path)
            output_path = f"{base}_{file_id}{ext}"

        output_file = output_dir / output_path

        # Log file sizes before writing
        self.logger.info(
            f"Writing repair result: {len(result.encode('utf-8'))} bytes to {output_file}"
        )

        # Final validation before write (no original_code available here)
        if self._check_file_completeness(
            result, original_code=None
        ):  # Double-check to be safe
            output_file.write_text(result)

            # Verify written file
            if output_file.exists():
                written_size = output_file.stat().st_size
                self.logger.info(f"Verified written file size: {written_size} bytes")

                if repair_time is not None:
                    self.logger.info(
                        f"Saved {repair_type} repair result to {output_file} after {repair_time:.2f}s"
                    )
                else:
                    self.logger.info(
                        f"Saved {repair_type} repair result to {output_file}"
                    )
        else:
            self.logger.error(
                f"Final validation failed - repair result became incomplete, skipping save"
            )

    def get_registry_info(self) -> str:
        """
        Get a string representation of the repair registry for debugging.

        Returns:
            String containing information about registered modules and error types
        """
        info = ["Repair Registry Information:"]
        info.append(f"- Number of registered modules: {len(self.repair_modules)}")

        # Module information
        info.append("\nRegistered Modules:")
        for name, module in self.repair_modules.items():
            info.append(f"  - {name}: {module.desc}")

        # Error type mappings
        info.append("\nError Type Mappings:")
        error_to_module_name = {}
        for error_type, module in self.error_to_module_map.items():
            if module.name not in error_to_module_name:
                error_to_module_name[module.name] = []
            error_to_module_name[module.name].append(error_type)

        for module_name, error_types in error_to_module_name.items():
            info.append(f"  - {module_name} handles:")
            for error_type in error_types:
                info.append(f"    - {error_type.name}")

        return "\n".join(info)

    def _try_fallback_repair(
        self,
        context,
        output_dir: Optional[Path] = None,
        max_attempts: int = 3,
        preserve_trial: bool = False,
    ) -> tuple[Optional[str], Optional[float]]:
        """
        Attempt fallback repair for compilation errors.

        Args:
            context: The execution context
            output_dir: Optional directory to save repair results
            max_attempts: Maximum number of fallback attempts
            preserve_trial: Whether to keep the failed trial in context

        Returns:
            Tuple of (repaired code if successful, score if successful)
        """
        if not context.trials:
            self.logger.warning("No trials available for fallback repair.")
            return None, None

        last_trial = context.trials[-1]
        if not last_trial.eval.compilation_error:
            self.logger.info("No compilation error detected.")
            return None, None

        # Store original state
        original_score = last_trial.eval.get_score()
        original_code = last_trial.code
        original_size = len(original_code.encode("utf-8"))
        self.logger.info(f"Original code size: {original_size} bytes")

        attempt = 0

        while attempt < max_attempts:
            attempt += 1
            self.logger.info(f"Fallback repair attempt {attempt}/{max_attempts}")

            # Check for modules registered to handle syntax errors
            syntax_modules = [
                m for m in self.repair_modules.values() if m.name == "repair_syntax"
            ]

            if not syntax_modules:
                self.logger.warning("No repair module found for compilation errors.")
                return None, None

            syntax_module = syntax_modules[0]
            self.logger.info(
                f"Attempting compilation error repair with {syntax_module.name}..."
            )

            # Try repair
            result = syntax_module.exec(context)
            if not result:
                self.logger.warning(f"Fallback attempt {attempt} produced no result.")
                continue

            # Validate size before evaluating
            if not self._check_file_size(result, original_size):
                self.logger.warning(
                    f"Fallback attempt {attempt} produced incomplete/invalid result"
                )
                continue

            # Evaluate result
            from src.modules.veval import VEval

            veval = VEval(result, self.logger)
            current_score = veval.eval_and_get_score()

            # Check if repair improved the score
            if current_score > original_score:
                self.logger.info(
                    f"Fallback attempt {attempt} improved score and passed size validation."
                )

                # Save result if directory provided
                if output_dir:
                    self._save_repair_result(
                        output_dir=output_dir,
                        output_path=f"fallback_result_{len(context.trials)}.rs",
                        result=result,
                        repair_type=f"fallback_attempt_{attempt}",
                    )

                return result, current_score

            self.logger.warning(f"Fallback attempt {attempt} did not improve score.")

        self.logger.warning(f"All {max_attempts} fallback attempts failed.")
        return None, None

    def repair_compilation_error(
        self, context, output_dir: Optional[Path] = None
    ) -> Optional[str]:
        """
        Handle compilation errors that may not have a specific VerusErrorType.
        This includes syntax errors and other compilation issues.

        Args:
            context: The execution context
            output_dir: Optional directory to save the repair result

        Returns:
            The repaired code if successful, None otherwise
        """
        result, _ = self._try_fallback_repair(
            context=context, output_dir=output_dir, max_attempts=3, preserve_trial=True
        )
        return result
