import os
import time
from pathlib import Path
from datetime import datetime

from loguru import logger

from configs.sconfig import config, reset_config
from context import Context, HyperParams, Trial
from modules.inv_inference import InvInferenceModule
from modules.progress_logger import ProgressLogger
from modules.repair_assertion import RepairAssertionModule
from modules.repair_postcond import RepairPostcondModule
from modules.repair_precond import RepairPrecondModule
from modules.repair_registry import RepairRegistry
from modules.spec_inference import SpecInferenceModule
from modules.veval import VerusErrorType, VEval, verus
from modules.view_inference import ViewInferenceModule
from modules.view_refinement import ViewRefinementModule

# Set the logging level to DEBUG to see more detailed information
logger.remove()
logger.add(lambda msg: print(msg, end=""), level="DEBUG")


def main():
    """
    Main entry point for VerusAgent
    """
    start_time = time.time()
    logger.info("Starting VerusAgent")

    # Use our custom config
    try:
        reset_config("config-azure")
        logger.info("Using config-azure configuration")

        # Set the verus path from the configuration
        if "verus_path" in config:
            verus.set_verus_path(config["verus_path"])
            logger.info(f"Verus path set to: {verus.verus_path}")
        else:
            logger.warning("verus_path not found in configuration")
    except Exception as e:
        logger.warning(f"Could not load config-azure or initialize verus path: {e}")
        logger.warning("Using default configuration")

    # Check for custom test file from environment variable
    custom_test_file = os.environ.get("VERUS_TEST_FILE")
    if custom_test_file:
        test_file_path = Path(custom_test_file)
        logger.info(f"Using custom test file from environment: {test_file_path}")
    else:
        # Default test file if no custom one specified
        test_file_path = Path("tests/rb_type_invariant_todo.rs")
        logger.info(f"Using default test file: {test_file_path}")
    
    if not test_file_path.exists():
        logger.error(f"Test file {test_file_path} not found!")
        return

    sample_code = test_file_path.read_text()
    logger.info(f"Loaded test file: {test_file_path}")

    # Create output directory if it doesn't exist
    output_dir = Path("output")
    output_dir.mkdir(exist_ok=True)

    # Create samples directory for intermediate results
    samples_dir = output_dir / "samples"
    samples_dir.mkdir(exist_ok=True)
    logger.info(f"Created directory for samples at {samples_dir.absolute()}")
    
    # Initialize the progress logger
    progress_logger = ProgressLogger(output_dir, logger)
    
    # Create a timestamp for this run to make all output files distinct
    run_timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    # Extract input file base name (without extension) for output files
    input_file_base = test_file_path.stem
    
    # Combine file identifiers for unique output filenames
    file_id = f"{input_file_base}_{run_timestamp}"
    logger.info(f"Output file identifier: {file_id}")
    
    # Set identifiers as environment variables for other modules to use
    os.environ["VERUS_RUN_TIMESTAMP"] = run_timestamp
    os.environ["VERUS_INPUT_FILE"] = input_file_base
    os.environ["VERUS_FILE_ID"] = file_id

    # Initialize context with sample code
    params = HyperParams()
    context = Context(sample_code, params, logger)

    # Initialize repair registry with all repair modules
    repair_registry = RepairRegistry.create(config, logger)

    # Log repair registry information in debug mode
    logger.debug(repair_registry.get_registry_info())

    # Register modules (inference, refinement, and repair)
    view_inference = ViewInferenceModule(config, logger)
    view_refinement = ViewRefinementModule(config, logger)
    inv_inference = InvInferenceModule(config, logger)
    spec_inference = SpecInferenceModule(config, logger)

    context.register_modoule("view_inference", view_inference)
    context.register_modoule("view_refinement", view_refinement)
    context.register_modoule("inv_inference", inv_inference)
    context.register_modoule("spec_inference", spec_inference)

    # Register all repair modules with the context
    repair_registry.register_with_context(context)

    logger.info(f"Registered modules: {list(context.modules.keys())}")

    # Run the entire workflow (Sequential for now, Planner integration is TODO)

    # Step 1: Generate View function
    progress_logger.start_step("view_inference", 1)
    step_start_time = time.time()
    logger.info("Step 1: Generating View function...")
    view_result = view_inference.exec(context)
    step_time = time.time() - step_start_time
    logger.info(f"View inference completed with result length: {len(view_result)} in {step_time:.2f}s")
    # Save the intermediate result with timestamp
    (output_dir / f"01_view_inference_{file_id}.rs").write_text(view_result)
    # Log step progress
    if context.trials and context.trials[-1].eval:
        progress_logger.end_step(context.trials[-1].eval.get_score(), len(view_result))

    # Step 2: Refine View function
    progress_logger.start_step("view_refinement", 2)
    step_start_time = time.time()
    logger.info("Step 2: Refining View function...")
    refined_view_result = view_refinement.exec(context)
    step_time = time.time() - step_start_time
    logger.info(
        f"View refinement completed with result length: {len(refined_view_result)} in {step_time:.2f}s"
    )
    # Save the intermediate result with timestamp
    (output_dir / f"02_view_refinement_{file_id}.rs").write_text(refined_view_result)
    # Log step progress
    if context.trials and context.trials[-1].eval:
        progress_logger.end_step(context.trials[-1].eval.get_score(), len(refined_view_result))

    # Step 3: Generate Inv function
    progress_logger.start_step("inv_inference", 3)
    step_start_time = time.time()
    logger.info("Step 3: Generating Inv function...")
    inv_result = inv_inference.exec(context)
    step_time = time.time() - step_start_time
    logger.info(f"Inv inference completed with result length: {len(inv_result)} in {step_time:.2f}s")
    # Save the intermediate result with timestamp
    (output_dir / f"03_inv_inference_{file_id}.rs").write_text(inv_result)
    # Log step progress
    if context.trials and context.trials[-1].eval:
        progress_logger.end_step(context.trials[-1].eval.get_score(), len(inv_result))

    # Step 4: Generate Requires/Ensures specifications
    progress_logger.start_step("spec_inference", 4)
    step_start_time = time.time()
    logger.info("Step 4: Generating Requires/Ensures specifications...")
    spec_result = spec_inference.exec(context)
    step_time = time.time() - step_start_time
    logger.info(f"Spec inference completed with result length: {len(spec_result)} in {step_time:.2f}s")
    # Save the intermediate result with timestamp
    (output_dir / f"04_spec_inference_{file_id}.rs").write_text(spec_result)
    # Log step progress
    if context.trials and context.trials[-1].eval:
        progress_logger.end_step(context.trials[-1].eval.get_score(), len(spec_result))

    # Step 5: Attempt repairs if needed using the repair registry
    last_trial = context.trials[-1]
    failures = last_trial.eval.get_failures()
    if failures:
        logger.info(f"Last trial has failures. Attempting repairs...")

        # Multiple rounds of repair
        max_repair_rounds = 7  # Maximum number of repair rounds to attempt
        current_round = 1
        previous_failure_count = len(failures)
        previous_verified_count = last_trial.eval.get_verified_count()
        previous_non_other_failures = sum(1 for failure in failures if failure.error.name != "Other")
        
        while failures and current_round <= max_repair_rounds:
            # Start repair round tracking
            progress_logger.start_repair_round(current_round)
            logger.info(f"Starting repair round {current_round}/{max_repair_rounds}")

            # Store the score before repairs
            before_score = last_trial.eval.get_score()
            
            # Track time for this repair round
            repair_round_start = time.time()
            
            # Use the repair registry to handle all failures
            repair_results = repair_registry.repair_all(context, failures, output_dir, progress_logger)

            # Calculate repair round time
            repair_round_time = time.time() - repair_round_start

            # Check if any repairs were successful
            if repair_results:
                logger.info(
                    f"Round {current_round}: Completed repairs for: {', '.join([err.name for err in repair_results.keys()])} in {repair_round_time:.2f}s"
                )
            else:
                logger.warning(f"Round {current_round}: No repairs were completed in {repair_round_time:.2f}s")
                progress_logger.end_repair_round()
                
                # For errors of type "Other", don't break out of the repair loop
                # Check if any "Other" errors are present
                has_other_errors = any(failure.error.name == "Other" for failure in failures)
                if has_other_errors:
                    logger.info("Errors of type 'Other' found. Continuing to next repair round...")
                    # Continue to the next round without breaking
                else:
                    break  # Exit if no repairs were made and no "Other" errors

            # Get the new failures after repairs
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures()
            current_failure_count = len(failures)
            current_verified_count = last_trial.eval.get_verified_count()
            
            # Count failures excluding "Other" errors
            current_non_other_failures = sum(1 for failure in failures if failure.error.name != "Other")
            
            # Check if we made progress (excluding "Other" errors from the comparison)
            if (
                # Only break if there are non-Other errors present AND no progress is made
                current_non_other_failures > 0 and
                current_non_other_failures >= previous_non_other_failures and
                current_verified_count <= previous_verified_count
            ):
                logger.info(
                    f"Round {current_round}: No progress made on non-Other errors (Non-Other failures: {current_non_other_failures}, Verified: {current_verified_count})"
                )
                progress_logger.end_repair_round()
                break  # Exit if no progress was made
            
            # Update counters for the next round
            previous_failure_count = current_failure_count
            previous_verified_count = current_verified_count
            previous_non_other_failures = current_non_other_failures
            
            # End the repair round tracking
            progress_logger.end_repair_round()
            
            current_round += 1

            # Save intermediate results after each round with timestamp
            round_result = context.trials[-1].code
            (output_dir / f"repair_round_{current_round-1}_{file_id}.rs").write_text(round_result)

            # Special handling for "Other" errors
            # If we only have "Other" errors remaining and repairs failed
            if failures and all(failure.error.name == "Other" for failure in failures) and not repair_results:
                logger.info("Only 'Other' type errors remain. Attempting fallback strategy...")
                
                # Find the best trial so far
                best_trial = None
                best_score = None
                
                for trial in context.trials:
                    if trial.eval and (best_score is None or trial.eval.get_score() > best_score):
                        best_score = trial.eval.get_score()
                        best_trial = trial
                
                if best_trial and best_trial != context.trials[-1]:
                    logger.info(f"Reverting to best trial with score: {best_score}")
                    
                    # Get the best code
                    best_code = best_trial.code
                    
                    # Create a VEval object first
                    v_eval = VEval(best_code, logger)
                    
                    # Create the fallback trial using the correct constructor parameters
                    trial_id = len(context.trials)
                    tmp_dir = config.get("tmp_dir", "tmp")
                    path = os.path.join(tmp_dir, f"trial_{trial_id}_fallback.rs")
                    
                    # Write the code to file
                    with open(path, "w") as f:
                        f.write(best_code)
                    
                    # Create the Trial object with the correct parameters
                    fallback_trial = Trial(trial_id, v_eval, path, logger)
                    
                    # Add to context
                    context.trials.append(fallback_trial)
                    
                    # Update failures for next round
                    failures = fallback_trial.eval.get_failures()
                    
                    # Log the fallback
                    logger.info(f"Fallback complete. New failure count: {len(failures)}")
                    
                    # Save the fallback result
                    (output_dir / f"fallback_result_{current_round-1}_{file_id}.rs").write_text(fallback_trial.code)

        if failures:
            logger.warning(
                f"Repairs completed after {current_round-1} rounds. {len(failures)} failures remain."
            )
        else:
            logger.info(f"All failures fixed after {current_round-1} repair rounds!")
    else:
        logger.info("No failures detected after inference. Skipping repair stage.")

    # Save the final result with timestamp and to a consistent file
    final_result = context.trials[-1].code
    (output_dir / f"final_result_{file_id}.rs").write_text(final_result)
    (output_dir / "final_result.rs").write_text(final_result)

    # Save the global best if available
    global_best_code = context.get_best_code()
    logger.debug(f"Main - Final global_best_code is None: {global_best_code is None}")

    if global_best_code:
        global_best_score = context.get_best_score()
        logger.debug(f"Main - Final global_best_score: {global_best_score}")

        # Save to output directory with timestamp
        global_best_path = output_dir / f"global_best_result_{file_id}.rs"
        global_best_with_score = (
            f"{global_best_code}\n\n// VEval Score: {global_best_score}"
        )
        global_best_path.write_text(global_best_with_score)
        logger.info(f"Saved global best result with score: {global_best_score}")

        # Also ensure it's saved to the best directory
        best_dir = Path("output/best")
        best_dir.mkdir(exist_ok=True, parents=True)
        best_file = best_dir / f"best_{file_id}.rs"
        best_file.write_text(global_best_with_score)
        # Also save a copy as just "best.rs" (overwriting previous)
        (best_dir / "best.rs").write_text(global_best_with_score)
        logger.info(f"Saved global best to {best_file}")

        # If the global best has a better score than the final result, use it as the final result
        final_score = context.trials[-1].eval.get_score()
        logger.debug(f"Main - Final trial score: {final_score}")
        
        # Record the final result in our progress logger
        progress_logger.record_final_result(final_score)
        
        # Compare the global best score with the final score
        # If global best is better, overwrite the final result
        if global_best_score > final_score:
            logger.info(
                f"Global best score ({global_best_score}) is better than final result ({final_score}). "
                f"Overwriting final result with global best."
            )
            # Write to both the timestamped file and the standard final_result.rs file
            (output_dir / f"final_result_{file_id}.rs").write_text(global_best_with_score)
            (output_dir / "final_result.rs").write_text(global_best_with_score)
            
            # Verify that the file was successfully overwritten
            final_result_path = output_dir / "final_result.rs"
            if final_result_path.exists():
                logger.info(f"Verified: Final result file was successfully overwritten with global best (size: {final_result_path.stat().st_size} bytes)")
            else:
                logger.warning("Failed to overwrite final_result.rs file")
                
            # Also update the final score for recording
            progress_logger.record_final_result(global_best_score)
        # Special check for compilation errors - always prefer code that compiles
        elif not global_best_score.compilation_error and final_score.compilation_error:
            logger.info(
                f"Global best compiles while final result has compilation errors. "
                f"Overwriting final result with global best."
            )
            # Write to both the timestamped file and the standard final_result.rs file
            (output_dir / f"final_result_{file_id}.rs").write_text(global_best_with_score)
            (output_dir / "final_result.rs").write_text(global_best_with_score)
            
            # Verify that the file was successfully overwritten
            final_result_path = output_dir / "final_result.rs"
            if final_result_path.exists():
                logger.info(f"Verified: Final result file was successfully overwritten with global best (size: {final_result_path.stat().st_size} bytes)")
            else:
                logger.warning("Failed to overwrite final_result.rs file")
                
            # Also update the final score for recording
            progress_logger.record_final_result(global_best_score)
        else:
            # Even if we're not overwriting with global best, still create a consistent final_result.rs file
            (output_dir / "final_result.rs").write_text(final_result)
    else:
        # Still record the final result even if no global best
        final_score = context.trials[-1].eval.get_score()
        progress_logger.record_final_result(final_score)
        
        logger.warning(
            "No global best code available. Check if global best tracking is working correctly."
        )

    total_time = time.time() - start_time
    logger.info(f"VerusAgent completed in {total_time:.2f}s! Results saved to {output_dir.absolute()}")


if __name__ == "__main__":
    main()
