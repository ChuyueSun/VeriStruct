import os
from pathlib import Path
from typing import Any, Dict, List, Optional

from infer import LLM
from modules.base import BaseModule
from modules.utils import (
    debug_type_error,
    evaluate_samples,
    save_selection_info,
    update_checkpoint_best,
)
from modules.veval import VEval
from prompts.template import build_instruction


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

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  4. Use a flattened tuple.
  5. Return the **entire updated Verus file** with your refined "View" function.

Please provide only the complete Rust code of the refined file with no additional commentary.
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

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.refinement_instruction,
            add_common=True,
            add_view=True,  # Include view refinement guidelines
            code=code
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
                temp=1.0,
            )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            # Return a placeholder response in case of error
            return code

        # Process responses to fix any type errors
        processed_responses = []
        for response in responses:
            # Apply debug_type_error to fix any type errors
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            if fixed_response:  # Only use the fixed version if it's not empty
                processed_responses.append(fixed_response)
            else:
                processed_responses.append(response)

        # Save all generated samples
        output_dir = Path("output/samples")
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = Path("output/best")
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate the samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses,
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