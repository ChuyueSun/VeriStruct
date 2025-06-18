import os
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    save_selection_info,
    update_checkpoint_best,
    get_examples,
    code_change_is_safe,
)
from src.modules.veval import VEval
from src.prompts.template import build_instruction
from src.utils.path_utils import samples_dir, best_dir


class ViewRefinementModule(BaseModule):
    """
    Module for refining View functions in Verus code.

    This module improves a View function by ensuring it provides
    a proper abstraction of the data structure.
    """

    def __init__(self, config, logger):
        """
        Initialize the ViewRefinementModule.

        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_refinement",
            desc="Refine an existing View function to improve its mathematical abstraction",
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)

        # Main instruction for view refinement
        self.refinement_instruction = """
You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "View" function within the given Verus file. The "View" function is the mathematical abstraction for a data structure, capturing the minimal information needed for its specification in Verus.

It is perfectly acceptable to leave the code unchanged if the current abstraction is already appropriate; modify the "View" function only when necessary.

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  4. Any refined view must convey at least the same amount of information while being more succinct. Aim to use a flattened tuple that is shorter than the original.
  5. Return the **entire updated Verus file** with your refined "View" function (or the original file if no changes were necessary) and nothing else changed.

Please provide only the complete Rust code of the file with no additional commentary.
"""

    def exec(self, context) -> str:
        """
        Execute the view refinement module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with refined View implementation
        """
        self.logger.info("View Refinement ...")

        # Get the latest trial code
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.refinement_instruction,
            add_common=True,
            add_view=True,  # Include view refinement guidelines
            code=code,
            knowledge=context.gen_knowledge(),
        )

        # Load examples
        examples = []
        try:
            example_path = (
                Path(self.config.get("example_path", "examples")) / "input-view-refine"
            )
            if example_path.exists():
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = (
                            Path(self.config.get("example_path", "examples"))
                            / "output-view-refine"
                            / f.name
                        )
                        answer = answer_path.read_text() if answer_path.exists() else ""
                        examples.append({"query": input_content, "answer": answer})
            else:
                self.logger.warning(
                    "Example path does not exist - proceeding without examples"
                )
        except Exception as e:
            self.logger.error(f"Error loading examples: {e}")

        # Retry mechanism for safety checks
        max_retries = 3
        safe_responses = []
        
        for retry_attempt in range(max_retries):
            self.logger.info(f"View refinement attempt {retry_attempt + 1}/{max_retries}")
            
            # Run inference
            try:
                responses = self.llm.infer_llm(
                    self.config.get("aoai_generation_model", "gpt-4"),
                    instruction,
                    examples,
                    code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 8192),
                    temp=1.0 + (retry_attempt * 0.2),  # Increase temperature on retries
                )
            except Exception as e:
                self.logger.error(f"Error during LLM inference: {e}")
                if retry_attempt == max_retries - 1:
                    # Last attempt failed, return original code
                    return code
                continue

            # Process responses to fix any type errors
            processed_responses = []
            for response in responses:
                # Apply debug_type_error to fix any type errors
                fixed_response, _ = debug_type_error(response, logger=self.logger)
                final_response = fixed_response if fixed_response else response
                
                # Check if the generated code is safe
                if self.check_code_safety(original_code, final_response):
                    processed_responses.append(final_response)
                    safe_responses.append(final_response)
                    self.logger.info("Generated view refinement code passed safety check")
                else:
                    self.logger.warning("Generated view refinement code failed safety check, will retry")

            # If we have safe responses, break out of retry loop
            if safe_responses:
                self.logger.info(f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts")
                break
            
            # If this is not the last attempt, modify instruction for retry
            if retry_attempt < max_retries - 1:
                instruction += f"\n\nIMPORTANT: Previous attempt failed safety checks. Please ensure your View refinement does not modify immutable functions and maintains semantic equivalence. Attempt {retry_attempt + 2}/{max_retries}."

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            safe_responses = [original_code]

        # Save all generated samples
        output_dir = samples_dir()

        # Create a directory for tracking global best samples
        global_dir = best_dir()

        # Evaluate the samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="02_view_refinement",
            logger=self.logger,
        )

        # Get the global best from context
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        # Update global best if current best is better, but don't use it for the current step
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best view refinement from this step to a module-specific file
        module_best_path = output_dir / "02_view_refinement_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view refinement to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view refinement: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context, regardless of global best
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code

    def check_code_safety(self, original_code: str, new_code: str) -> bool:
        """
        Check if code changes are safe using Lynette comparison.
        
        Args:
            original_code: Original code
            new_code: Modified code
            
        Returns:
            True if changes are safe, False otherwise
        """
        try:
            # Get immutable functions from config if available
            immutable_funcs = self.config.get("immutable_functions", [])
            
            return code_change_is_safe(
                origin_code=original_code,
                changed_code=new_code,
                verus_path=self.config.get("verus_path", "verus"),
                logger=self.logger,
                immutable_funcs=immutable_funcs
            )
        except Exception as e:
            self.logger.error(f"Error checking code safety: {e}")
            return True  # Default to safe if check fails
