import loguru
import os
from pathlib import Path
from context import Trial, Context, HyperParams
from modules.view_inference import ViewInferenceModule
from modules.view_refinement import ViewRefinementModule
from modules.inv_inference import InvInferenceModule
from configs.sconfig import config, reset_config

logger = loguru.logger

def main():
    """
    Main entry point for VerusAgent
    """
    logger.info("Starting VerusAgent")
    
    # Use our custom config
    try:
        reset_config("config-verusagent")
        logger.info("Using config-verusagent configuration")
    except:
        logger.warning("Could not load config-verusagent, using default configuration")
    
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
    
    # Initialize context with sample code
    params = HyperParams()
    context = Context(sample_code, params, logger)
    
    # Register modules
    view_inference = ViewInferenceModule(config, logger)
    view_refinement = ViewRefinementModule(config, logger)
    inv_inference = InvInferenceModule(config, logger)
    
    context.register_modoule("view_inference", view_inference)
    context.register_modoule("view_refinement", view_refinement)
    context.register_modoule("inv_inference", inv_inference)
    
    logger.info(f"Registered modules: {list(context.modules.keys())}")
    
    # Run the entire workflow
    
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
    
    # Save the final result
    final_result = context.trials[-1].code
    (output_dir / "final_result.rs").write_text(final_result)
    
    logger.info(f"VerusAgent completed! Results saved to {output_dir.absolute()}")

if __name__ == "__main__":
    main()
