import loguru
import os
from pathlib import Path
from context import Trial, Context, HyperParams
from modules.view_inference import ViewInferenceModule
from modules.view_refinement import ViewRefinementModule
from modules.inv_inference import InvInferenceModule
from modules.repair_assertion import RepairAssertionModule
from modules.repair_precond import RepairPrecondModule
from modules.repair_postcond import RepairPostcondModule
from configs.sconfig import config, reset_config
from modules.veval import verus, VEval, VerusErrorType

logger = loguru.logger
# Set the logging level to DEBUG to see more detailed information
logger.remove()
logger.add(lambda msg: print(msg, end=""), level="DEBUG")

def main():
    """
    Main entry point for VerusAgent
    """
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
    
    # Load the RingBuffer example from tests/rb_type_invariant_todo.rs
    test_file_path = Path("tests/rb_type_invariant_todo.rs")
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
    
    # Initialize context with sample code
    params = HyperParams()
    context = Context(sample_code, params, logger)
    
    # Register modules (inference, refinement, and repair)
    view_inference = ViewInferenceModule(config, logger)
    view_refinement = ViewRefinementModule(config, logger)
    inv_inference = InvInferenceModule(config, logger)
    repair_assertion = RepairAssertionModule(config, logger)
    repair_precond = RepairPrecondModule(config, logger)
    repair_postcond = RepairPostcondModule(config, logger)
    
    context.register_modoule("view_inference", view_inference)
    context.register_modoule("view_refinement", view_refinement)
    context.register_modoule("inv_inference", inv_inference)
    context.register_modoule("repair_assertion", repair_assertion)
    context.register_modoule("repair_precond", repair_precond)
    context.register_modoule("repair_postcond", repair_postcond)
    
    logger.info(f"Registered modules: {list(context.modules.keys())}")
    
    # Run the entire workflow (Sequential for now, Planner integration is TODO)
    
    # Step 1: Generate View function
    logger.info("Step 1: Generating View function...")
    view_result = view_inference.exec(context)
    logger.info(f"View inference completed with result length: {len(view_result)}")
    # Save the intermediate result
    (output_dir / "01_view_inference.rs").write_text(view_result)
    
    # Step 2: Refine View function
    logger.info("Step 2: Refining View function...")
    refined_view_result = view_refinement.exec(context)
    logger.info(f"View refinement completed with result length: {len(refined_view_result)}")
    # Save the intermediate result
    (output_dir / "02_view_refinement.rs").write_text(refined_view_result)
    
    # Step 3: Generate Inv function
    logger.info("Step 3: Generating Inv function...")
    inv_result = inv_inference.exec(context)
    logger.info(f"Inv inference completed with result length: {len(inv_result)}")
    # Save the final result
    (output_dir / "03_inv_inference.rs").write_text(inv_result)
    
    # Step 4: Attempt repairs if needed (Example: try fixing assertion error)
    last_trial = context.trials[-1]
    failures = last_trial.eval.get_failures()
    if failures:
        logger.info(f"Last trial has failures. Attempting repairs...")
        
        # Create a mapping from error types to repair modules and output filenames
        repair_mapping = {
            VerusErrorType.AssertFail: (repair_assertion, "04_repair_assertion.rs"),
            VerusErrorType.PreCondFail: (repair_precond, "05_repair_precond.rs"),
            VerusErrorType.PostCondFail: (repair_postcond, "06_repair_postcond.rs"),
        }
        
        # Process each error type in the mapping
        for error_type, (repair_module, output_filename) in repair_mapping.items():
            # Find failures of this type
            type_failures = [f for f in failures if f.error == error_type]
            if type_failures:
                logger.info(f"Attempting {error_type.name} repair...")
                repair_result = repair_module.exec(context, type_failures[0])
                (output_dir / output_filename).write_text(repair_result)
                logger.info(f"{error_type.name} repair completed.")

    # Save the final result (potentially after repairs)
    final_result = context.trials[-1].code
    (output_dir / "final_result.rs").write_text(final_result)
    
    # Save the global best if available
    global_best_code = context.get_best_code()
    logger.debug(f"Main - Final global_best_code is None: {global_best_code is None}")
    
    if global_best_code:
        global_best_score = context.get_best_score()
        logger.debug(f"Main - Final global_best_score: {global_best_score}")
        
        # Save to output directory
        global_best_path = output_dir / "global_best_result.rs"
        global_best_with_score = f"{global_best_code}\n\n// VEval Score: {global_best_score}"
        global_best_path.write_text(global_best_with_score)
        logger.info(f"Saved global best result with score: {global_best_score}")
        
        # Also ensure it's saved to the best directory
        best_dir = Path("output/best")
        best_dir.mkdir(exist_ok=True, parents=True)
        best_file = best_dir / "best.rs"
        best_file.write_text(global_best_with_score)
        logger.info(f"Saved global best to {best_file}")

        # If the global best has a better score than the final result, use it as the final result
        final_score = context.trials[-1].eval.get_score()
        logger.debug(f"Main - Final trial score: {final_score}")
        if global_best_score and global_best_score.is_correct() and not final_score.is_correct():
            logger.info("Global best is correct while final result is not. Overwriting final result with global best.")
            (output_dir / "final_result.rs").write_text(global_best_with_score)
    else:
        logger.warning("No global best code available. Check if global best tracking is working correctly.")
    
    logger.info(f"VerusAgent completed! Results saved to {output_dir.absolute()}")

if __name__ == "__main__":
    main()
