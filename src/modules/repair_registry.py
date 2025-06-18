"""
Registry for repair modules in VerusAgent.
Maps error types to appropriate repair modules.
"""

import logging
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Type

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
        from src.modules.repair_postcond import RepairPostcondModule
        from src.modules.repair_precond import RepairPrecondModule
        from src.modules.repair_remove_inv import RepairRemoveInv
        from src.modules.repair_syntax import RepairSyntaxModule
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

        # Initialize and register assertion repair module
        assertion_repair = RepairAssertionModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_assertion",
            assertion_repair,
            [VerusErrorType.AssertFail, VerusErrorType.SplitAssertFail],
            "04_repair_assertion.rs",
        )

        # Initialize and register precondition repair module
        precond_repair = RepairPrecondModule(config, logger, immutable_funcs)
        registry.register_module(
            "repair_precond",
            precond_repair,
            [
                VerusErrorType.PreCondFail,
                VerusErrorType.PreCondFailVecLen,
                VerusErrorType.SplitPreFail,
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
                VerusErrorType.SplitPostFail,
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
            [VerusErrorType.CannotCallFunc],
            "12_repair_mode.rs",
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
            VerusErrorType.SplitAssertFail: 14,  # Fix split assertion failures
            VerusErrorType.PreCondFail: 15,  # Fix precondition failures
            VerusErrorType.SplitPreFail: 16,  # Fix split precondition failures
            VerusErrorType.PostCondFail: 17,  # Fix postcondition failures
            VerusErrorType.SplitPostFail: 18,  # Fix split postcondition failures
            VerusErrorType.ensure_private: 19,  # Fix private field access in ensures
            VerusErrorType.require_private: 20,  # Fix private function access in requires
            VerusErrorType.RecommendNotMet: 21,  # Fix recommendation not met errors
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
    ) -> Dict[VerusErrorType, str]:
        """
        Attempt to repair all errors in the list using appropriate modules.

        Args:
            context: The execution context
            failures: List of errors to repair
            output_dir: Optional directory to save repair results
            progress_logger: Optional progress logger to track repair operations

        Returns:
            Dictionary mapping error types to repaired code
        """
        result_map = {}

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
                self.logger.info("Compilation error with no specialised handler – attempting syntax repair…")

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
                                self.logger.info("All errors fixed after compilation repair.")
                                result_map["compilation"] = compilation_result
                                return result_map
                    else:
                        self.logger.warning("Syntax repair did not improve score – skipping.")
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

                # Use the first failure of this type
                result = module.exec(context, type_failures[0])

                # Calculate repair time
                repair_time = time.time() - repair_start_time

                # Get the trial that was added by the repair module
                if context.trials and len(context.trials) > 0:
                    after_score = context.trials[-1].eval.get_score()

                    # Check if the repair improved the score
                    if after_score <= before_score:
                        self.logger.warning(
                            f"{error_type.name} repair did not improve the score or made it worse."
                        )
                        # If repair made things worse (especially causing compilation errors),
                        # remove the last trial to revert to the previous state
                        if (
                            context.trials[-1].eval.compilation_error
                            and not before_score.compilation_error
                        ):
                            self.logger.warning(
                                "Repair introduced compilation errors. Reverting to previous state."
                            )
                            context.trials.pop()  # Remove the last trial
                            # Skip adding this error type to result_map
                            continue
                    else:
                        # Only add to result_map if repair was successful
                        result_map[error_type] = result
                        made_progress = True

                        # Save the result if an output directory is provided
                        if output_dir and result:
                            output_path = self.get_output_path(type_failures[0])
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
                                    f"Saved {error_type.name} repair result to {output_file} after {repair_time:.2f}s"
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
            else:
                # For 'Other' error type, log a warning but don't terminate repair process
                if error_type.name == "Other":
                    self.logger.warning(
                        f"No repair module registered for error type: {error_type.name} - continuing with other errors"
                    )
                else:
                    self.logger.warning(
                        f"No repair module registered for error type: {error_type.name}"
                    )

        # If we made progress on at least some errors, return the results
        # even if we couldn't repair all errors
        return result_map

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
        last_trial = context.trials[-1]

        if not last_trial.eval.compilation_error:
            self.logger.info("No compilation error detected.")
            return None

        # Check for modules registered to handle syntax errors
        syntax_modules = [
            m for m in self.repair_modules.values() if m.name == "repair_syntax"
        ]

        if syntax_modules:
            syntax_module = syntax_modules[0]
            self.logger.info(
                f"Attempting compilation error repair with {syntax_module.name}..."
            )
            result = syntax_module.exec(context)

            if output_dir and result:
                # Get file ID from environment (set in main.py)
                file_id = os.environ.get("VERUS_FILE_ID", "")
                output_path = "03_repair_syntax.rs"

                if file_id:
                    # Insert file ID before file extension
                    base, ext = os.path.splitext(output_path)
                    output_path = f"{base}_{file_id}{ext}"

                output_file = output_dir / output_path
                output_file.write_text(result)
                self.logger.info(f"Saved syntax repair result to {output_file}")

            return result

        self.logger.warning("No repair module found for compilation errors.")
        return None
