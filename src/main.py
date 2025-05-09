import os
import time
from datetime import datetime
from pathlib import Path

from loguru import logger

from src.configs.sconfig import config, reset_config
from src.context import Context, HyperParams, Trial
from src.modules.inv_inference import InvInferenceModule
from src.modules.progress_logger import ProgressLogger
from src.modules.repair_assertion import RepairAssertionModule
from src.modules.repair_postcond import RepairPostcondModule
from src.modules.repair_precond import RepairPrecondModule
from src.modules.repair_registry import RepairRegistry
from src.modules.spec_inference import SpecInferenceModule
from modules.veval import VerusErrorType, VEval, verus
from src.modules.utils import parse_plan_execution_order
from src.modules.view_inference import ViewInferenceModule
from src.modules.view_refinement import ViewRefinementModule
from src.planner import Planner

# Set the logging level to DEBUG to see more detailed information
logger.remove()
logger.add(lambda msg: print(msg, end=""), level="DEBUG")


def write_and_verify_file(file_path: Path, content: str, logger) -> bool:
    """Helper function to write content to a file and verify the write was successful."""
    file_path.write_text(content)
    if file_path.exists():
        logger.info(
            f"Verified: File successfully written to {file_path} (size: {file_path.stat().st_size} bytes)"
        )
        return True
    else:
        logger.warning(f"Failed to write file: {file_path}")
        return False


def handle_checkpoint_best(context, output_dir, file_id, progress_logger, logger):
    """Handle the checkpoint best code and score logic."""
    checkpoint_best_code = context.get_best_code()
    logger.debug(f"Main - Final checkpoint_best_code is None: {checkpoint_best_code is None}")

    if not checkpoint_best_code:
        final_score = context.trials[-1].eval.get_score()
        progress_logger.record_final_result(final_score)
        logger.warning(
            "No checkpoint best code available. Check if checkpoint best tracking is working correctly."
        )
        return

    checkpoint_best_score = context.get_best_score()
    logger.debug(f"Main - Final checkpoint_best_score: {checkpoint_best_score}")

    # Save to output directory with timestamp
    checkpoint_best_path = output_dir / f"checkpoint_best_{file_id}.rs"
    # Add detailed score information at the end of the file
    checkpoint_best_with_score = (
        f"{checkpoint_best_code}\n\n"
        f"// Checkpoint Best VEval Score: {checkpoint_best_score}\n"
        f"// Verified: {checkpoint_best_score.verified}, Errors: {checkpoint_best_score.errors}, Verus Errors: {checkpoint_best_score.verus_errors}\n"
        f"// Compilation Error: {checkpoint_best_score.compilation_error}"
    )
    write_and_verify_file(checkpoint_best_path, checkpoint_best_with_score, logger)
    logger.info(f"Saved checkpoint best result to {checkpoint_best_path} with score: {checkpoint_best_score}")

    # Save to best directory
    best_dir = Path("output/best")
    best_dir.mkdir(exist_ok=True, parents=True)
    best_file = best_dir / f"best_{file_id}.rs"
    write_and_verify_file(best_file, checkpoint_best_with_score, logger)
    write_and_verify_file(best_dir / "best.rs", checkpoint_best_with_score, logger)
    logger.info(f"Saved checkpoint best to {best_file}")

    # Compare with final result
    final_score = context.trials[-1].eval.get_score()
    logger.debug(f"Main - Final trial score: {final_score}")
    progress_logger.record_final_result(final_score)

    should_use_checkpoint_best = checkpoint_best_score > final_score or (
        not checkpoint_best_score.compilation_error and final_score.compilation_error
    )

    if should_use_checkpoint_best:
        reason = (
            f"Checkpoint best score ({checkpoint_best_score}) is better than final result ({final_score})"
            if checkpoint_best_score > final_score
            else "Checkpoint best compiles while final result has compilation errors"
        )
        logger.info(f"{reason}. Overwriting final result with checkpoint best.")

        write_and_verify_file(
            output_dir / f"final_result_{file_id}.rs", checkpoint_best_with_score, logger
        )
        write_and_verify_file(
            output_dir / "final_result.rs", checkpoint_best_with_score, logger
        )
        progress_logger.record_final_result(checkpoint_best_score)
    else:
        write_and_verify_file(
            output_dir / "final_result.rs", context.trials[-1].code, logger
        )


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
        if os.environ.get("VERUS_PATH"):
            verus_path = os.environ.get("VERUS_PATH")
            verus.set_verus_path(verus_path)
            logger.info(f"Verus path set to: {verus.verus_path}")
            logger.info(f"VERUS_PATH environment variable used: {verus_path}")
        elif "verus_path" in config:
            verus.set_verus_path(config["verus_path"])
            logger.info(f"Verus path set to: {verus.verus_path}")
            # Also set as environment variable for modules to access
            os.environ["VERUS_PATH"] = str(config["verus_path"])
            logger.info(f"VERUS_PATH environment variable set to: {os.environ['VERUS_PATH']}")
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
    
    # Extract verification type information by analyzing the file name
    verification_type = ""
    if "type_invariant" in input_file_base:
        verification_type = "TypeInv"
    elif "postcond" in input_file_base:
        verification_type = "PostCond" 
    elif "precond" in input_file_base:
        verification_type = "PreCond"
    elif "invariant" in input_file_base:
        verification_type = "Invariant"
    elif "assertion" in input_file_base:
        verification_type = "Assert"
    elif "decrease" in input_file_base:
        verification_type = "Decrease"
    else:
        verification_type = "General"
        
    # For additional context, check for specific data structures in the filename
    data_structure = ""
    if "rb" in input_file_base:
        data_structure = "RB"  # Red-Black tree
    elif "vec" in input_file_base:
        data_structure = "Vec"  # Vector
    elif "list" in input_file_base:
        data_structure = "List"  # Linked list
    elif "map" in input_file_base:
        data_structure = "Map"  # Map/Dictionary
    elif "tree" in input_file_base:
        data_structure = "Tree"  # Tree structure
    
    # Combine file identifiers for unique and informative output filenames
    file_id = f"{data_structure}_{verification_type}_{run_timestamp}"
    logger.info(f"Output file identifier: {file_id} (from {input_file_base})")

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

    # Create and execute planner to get a workflow strategy
    logger.info("Creating verification plan using the Planner...")
    planner = Planner(logger)
    plan_result = planner.exec(context)
    logger.info(f"Planning complete. Plan length: {len(plan_result) if isinstance(plan_result, (list, tuple)) else 'unknown'}")
    
    # Extract the plan text from the result, handling various possible data structures
    def extract_text_from_data(data):
        if isinstance(data, str):
            return data
        elif isinstance(data, (list, tuple)):
            if len(data) == 0:
                return ""
            
            # Try the first element
            first_item = data[0]
            if isinstance(first_item, str):
                return first_item
            elif isinstance(first_item, (list, tuple, dict)):
                return extract_text_from_data(first_item)
        elif isinstance(data, dict):
            # If there's a 'content' key, use that
            if 'content' in data:
                return extract_text_from_data(data['content'])
            # Otherwise, just use the first value
            elif data:
                return extract_text_from_data(next(iter(data.values())))
        
        # Fallback: convert to string
        return str(data)
    
    plan_text = extract_text_from_data(plan_result)
    if not plan_text:
        plan_text = "No plan generated. Proceeding with default execution order."
        logger.warning(plan_text)
    
    # Save the plan to the output directory
    plan_file_path = output_dir / f"verification_plan_{file_id}.txt"
    write_and_verify_file(plan_file_path, plan_text, logger)
    logger.info(f"Saved verification plan to {plan_file_path}")
    
    # Add the plan to the context as knowledge
    context.add_knowledge("verification_plan", plan_text)
    logger.info("Added verification plan to context knowledge")

    # Parse the plan to determine execution order using the new utility function
    available_modules = list(context.modules.keys())
    execution_order = parse_plan_execution_order(plan_text, available_modules, logger)
    logger.info(f"Determined execution order from plan: {execution_order}")
    
    # Execute modules according to the plan-derived order
    step_number = 1
    for module_name in execution_order:
        # Ensure the module exists
        if module_name not in context.modules:
            logger.warning(f"Module '{module_name}' not found in registered modules. Skipping.")
            continue
            
        module = context.modules[module_name]
        
        # Start step tracking
        progress_logger.start_step(module_name, step_number)
        step_start_time = time.time()
        
        logger.info(f"Step {step_number}: Executing {module_name}...")
        step_result = module.exec(context)
        
        step_time = time.time() - step_start_time
        logger.info(f"{module_name} completed with result length: {len(step_result)} in {step_time:.2f}s")
        
        # Save the intermediate result with timestamp
        step_output_path = output_dir / f"{step_number:02}_{module_name}_{file_id}.rs"
        
        # Add score information if available
        if context.trials and context.trials[-1].eval:
            step_score = context.trials[-1].eval.get_score()
            step_result_with_score = (
                f"{step_result}\n\n"
                f"// Step {step_number} ({module_name}) VEval Score: {step_score}\n"
                f"// Verified: {step_score.verified}, Errors: {step_score.errors}, Verus Errors: {step_score.verus_errors}"
            )
            write_and_verify_file(step_output_path, step_result_with_score, logger)
        else:
            write_and_verify_file(step_output_path, step_result, logger)
            
        logger.info(f"Step {step_number} output saved to {step_output_path}")
        
        # Log step progress
        if context.trials and context.trials[-1].eval:
            progress_logger.end_step(context.trials[-1].eval.get_score(), len(step_result))
            
        step_number += 1

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
        previous_non_other_failures = sum(
            1 for failure in failures if failure.error.name != "Other"
        )
        
        # Track rounds where repair made things worse for fallback logic
        rounds_without_improvement = 0
        best_repair_score = last_trial.eval.get_score()
        best_repair_code = last_trial.code
        original_score = last_trial.eval.get_score()
        original_code = last_trial.code

        while failures and current_round <= max_repair_rounds:
            # Start repair round tracking
            progress_logger.start_repair_round(current_round)
            logger.info(f"Starting repair round {current_round}/{max_repair_rounds}")

            # Store the score before repairs
            before_score = last_trial.eval.get_score()

            # Track time for this repair round
            repair_round_start = time.time()

            # Use the repair registry to handle all failures
            repair_results = repair_registry.repair_all(
                context, failures, output_dir, progress_logger
            )

            # Calculate repair round time
            repair_round_time = time.time() - repair_round_start

            # Check if any repairs were successful
            if repair_results:
                logger.info(
                    f"Round {current_round}: Completed repairs for: {', '.join([err.name for err in repair_results.keys()])} in {repair_round_time:.2f}s"
                )
            else:
                logger.warning(
                    f"Round {current_round}: No repairs were completed in {repair_round_time:.2f}s"
                )
                progress_logger.end_repair_round()
                logger.info(
                    "Continuing to next repair round even though no repairs were made..."
                )

            # Get the new failures after repairs
            last_trial = context.trials[-1]
            failures = last_trial.eval.get_failures()
            current_failure_count = len(failures)
            current_verified_count = last_trial.eval.get_verified_count()
            current_score = last_trial.eval.get_score()

            # Count failures excluding "Other" errors
            current_non_other_failures = sum(
                1 for failure in failures if failure.error.name != "Other"
            )

            # Check if the current score is better than our best repair score
            if current_score > best_repair_score:
                best_repair_score = current_score
                best_repair_code = last_trial.code
                rounds_without_improvement = 0
                logger.info(f"Round {current_round}: Found better repair with score: {best_repair_score}")
            else:
                rounds_without_improvement += 1
                logger.info(f"Round {current_round}: No improvement in score. Rounds without improvement: {rounds_without_improvement}")

            # Check if we made progress (excluding "Other" errors from the comparison)
            if (
                current_non_other_failures > 0
                and current_non_other_failures >= previous_non_other_failures
                and current_verified_count <= previous_verified_count
            ):
                logger.info(
                    f"Round {current_round}: No progress made on non-Other errors (Non-Other failures: {current_non_other_failures}, Verified: {current_verified_count})"
                )
                logger.info("Continuing to next repair round despite no progress...")

            # Update counters for the next round
            previous_failure_count = current_failure_count
            previous_verified_count = current_verified_count
            previous_non_other_failures = current_non_other_failures

            # End the repair round tracking
            progress_logger.end_repair_round()

            current_round += 1

            # Save intermediate results after each round with timestamp
            round_result = context.trials[-1].code
            round_score = context.trials[-1].eval.get_score()
            
            # Add the score as a comment at the end of the file
            round_result_with_score = (
                f"{round_result}\n\n"
                f"// Repair Round {current_round-1} VEval Score: {round_score}\n"
                f"// Verified: {round_score.verified}, Errors: {round_score.errors}, Verus Errors: {round_score.verus_errors}"
            )
            
            repair_round_path = output_dir / f"repair_round_{current_round-1}_{file_id}.rs"
            write_and_verify_file(repair_round_path, round_result_with_score, logger)
            logger.info(f"Repair round {current_round-1} result saved to {repair_round_path}")

            # After three consecutive rounds with no improvement and score worse than original,
            # fallback to the best repair we've seen
            if rounds_without_improvement >= 3 and best_repair_score < original_score:
                logger.info(
                    f"No improvement for {rounds_without_improvement} consecutive rounds and score ({best_repair_score}) "
                    f"is worse than original ({original_score}). Reverting to best repair found."
                )
                
                # Create a VEval object first
                v_eval = VEval(best_repair_code, logger)

                # Create the fallback trial
                trial_id = len(context.trials)
                tmp_dir = config.get("tmp_dir", "tmp")
                path = os.path.join(tmp_dir, f"trial_{trial_id}_fallback.rs")

                # Write the code to file
                write_and_verify_file(Path(path), best_repair_code, logger)

                # Create the Trial object with the correct parameters
                fallback_trial = Trial(trial_id, v_eval, path, logger)

                # Add to context
                context.trials.append(fallback_trial)

                # Update failures for next round
                failures = fallback_trial.eval.get_failures()

                # Log the fallback
                logger.info(
                    f"Fallback complete. New failure count: {len(failures)}"
                )

                # Save the fallback result
                fallback_code = fallback_trial.code
                fallback_score = fallback_trial.eval.get_score()
                
                # Add the score as a comment at the end of the file
                fallback_with_score = (
                    f"{fallback_code}\n\n"
                    f"// Fallback VEval Score: {fallback_score}\n"
                    f"// Verified: {fallback_score.verified}, Errors: {fallback_score.errors}, Verus Errors: {fallback_score.verus_errors}"
                )
                
                fallback_path = output_dir / f"fallback_result_{current_round-1}_{file_id}.rs"
                write_and_verify_file(fallback_path, fallback_with_score, logger)
                logger.info(f"Fallback result saved to {fallback_path}")
                
                # Reset the rounds without improvement counter
                rounds_without_improvement = 0

            # Special handling for "Other" errors
            if (
                failures
                and all(failure.error.name == "Other" for failure in failures)
                and not repair_results
            ):
                logger.info(
                    "Only 'Other' type errors remain. Attempting fallback strategy..."
                )

                # Find the best trial so far
                best_trial = None
                best_score = None

                for trial in context.trials:
                    if trial.eval and (
                        best_score is None or trial.eval.get_score() > best_score
                    ):
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
                    write_and_verify_file(Path(path), best_code, logger)

                    # Create the Trial object with the correct parameters
                    fallback_trial = Trial(trial_id, v_eval, path, logger)

                    # Add to context
                    context.trials.append(fallback_trial)

                    # Update failures for next round
                    failures = fallback_trial.eval.get_failures()

                    # Log the fallback
                    logger.info(
                        f"Fallback complete. New failure count: {len(failures)}"
                    )

                    # Save the fallback result
                    fallback_code = fallback_trial.code
                    fallback_score = fallback_trial.eval.get_score()
                    
                    # Add the score as a comment at the end of the file
                    fallback_with_score = (
                        f"{fallback_code}\n\n"
                        f"// Fallback VEval Score: {fallback_score}\n"
                        f"// Verified: {fallback_score.verified}, Errors: {fallback_score.errors}, Verus Errors: {fallback_score.verus_errors}"
                    )
                    
                    fallback_path = output_dir / f"fallback_result_{current_round-1}_{file_id}.rs"
                    write_and_verify_file(fallback_path, fallback_with_score, logger)
                    logger.info(f"Fallback result saved to {fallback_path}")

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
    final_score = context.trials[-1].eval.get_score()
    
    # Add the score as a comment at the end of the file
    final_result_with_score = f"{final_result}\n\n// Final VEval Score: {final_score}\n// Verified: {final_score.verified}, Errors: {final_score.errors}, Verus Errors: {final_score.verus_errors}"
    
    final_result_path = output_dir / f"final_result_{file_id}.rs"
    write_and_verify_file(final_result_path, final_result_with_score, logger)
    logger.info(f"Final verification result saved to {final_result_path}")
    
    consistent_result_path = output_dir / "final_result.rs"
    write_and_verify_file(consistent_result_path, final_result_with_score, logger)
    logger.info(f"Latest result also saved to {consistent_result_path}")

    # Handle checkpoint best code and score
    handle_checkpoint_best(context, output_dir, file_id, progress_logger, logger)

    total_time = time.time() - start_time
    logger.info(
        f"VerusAgent completed in {total_time:.2f}s! Results saved to {output_dir.absolute()}"
    )


if __name__ == "__main__":
    main()
