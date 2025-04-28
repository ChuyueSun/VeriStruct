import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from infer import LLM
from modules.base import BaseModule
from modules.utils import (
    debug_type_error,
    evaluate_samples,
    parse_llm_response,
    save_selection_info,
    update_global_best,
)
from modules.veval import VEval
from prompts.template import build_instruction


class ViewInferenceModule(BaseModule):
    """
    Module for View function inference in Verus code.

    This module generates a View function that provides a mathematical abstraction
    for the given data structure, which is used in Verus specifications.
    """

    def __init__(self, config, logger):
        """
        Initialize the ViewInferenceModule.

        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_inference",
            desc="Generate a View function for the data structure's mathematical abstraction",
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)

        # Main instruction for View inference
        self.view_instruction = """
You are an expert in Verus (verifier for rust). Your task is to generate a View function for the given module. View is the mathematical abstraction for the given data structure. It contains the minimal information to completely represent it. View is used strictly in Verus spec.
    - Add a `View` spec function that provides a mathematical abstraction for types used in the executable code.
    - For `Vec` type variables in the `View`, append "@" to their names.
    - Fill in `/* TODO: part of view */`.
Mathematical types in Verus include:
    - bool
    - int
    - nat
    - Seq<T>
    - Set<T>
    - Map<K, V>

Steps:
    1. Infer the information should be contained in the return type of the `View` function. It could be any of the mathematical types mentioned above or a combination (tuple) of them.
    2. Generate the view function based on the inferred information. Return it as part of the input file.


Format:
```verus

impl<T: Copy> View for RingBuffer<T> {
    type V = // your inferred View return type here that contain the minimal information to represent the class

    closed spec fn view(&self) -> Self::V {
        ... // your implementation here
    }
}
```"""

    def parse_view_response(self, response: str) -> str:
        """
        Parse the LLM response to extract and clean the View implementation.

        Args:
            response: Raw LLM response text

        Returns:
            Cleaned code with proper View implementation
        """
        self.logger.info("Parsing view inference response...")

        # Use the general parser first to extract all Rust code
        parsed_code = parse_llm_response(response, logger=self.logger)

        # If parsing failed or returned empty string, log warning and return original
        if not parsed_code:
            self.logger.warning(
                "General parser couldn't extract code, using original response"
            )
            return response

        # Check if the parser gave us a complete View implementation
        if (
            "impl" in parsed_code
            and "View for" in parsed_code
            and "type V =" in parsed_code
        ):
            self.logger.info("Successfully extracted View implementation")
            return parsed_code

        # If we don't have a View implementation yet, try to extract it specifically
        view_impl_pattern = r"impl\s*<.*?>\s*View\s+for\s+\w+.*?{.*?type\s+V\s*=.*?closed\s+spec\s+fn\s+view.*?}.*?}"
        view_impls = re.findall(view_impl_pattern, parsed_code, re.DOTALL)

        if view_impls:
            self.logger.info("Extracted specific View implementation from parsed code")
            return view_impls[0]

        # If we still don't have a View implementation, try the original response
        view_impls = re.findall(view_impl_pattern, response, re.DOTALL)
        if view_impls:
            self.logger.info("Extracted View implementation from original response")
            return view_impls[0]

        # If nothing worked, return the parsed code anyway
        self.logger.warning(
            "Could not find specific View implementation, returning general parsed code"
        )
        return parsed_code

    def exec(self, context) -> str:
        """
        Execute the view inference module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with View function
        """
        self.logger.info("View Inference ...")

        # Get the latest trial code
        code = context.trials[-1].code

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.view_instruction,
            add_common=True,
            add_view=True,  # Include View guidelines
            code=code,
        )

        # Load examples
        examples = []
        try:
            example_path = (
                Path(self.config.get("example_path", "examples")) / "input-view"
            )
            if example_path.exists():
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = (
                            Path(self.config.get("example_path", "examples"))
                            / "output-view"
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

        # Parse and process responses
        processed_responses = []
        for response in responses:
            # First parse the response to extract the View implementation
            parsed_response = self.parse_view_response(response)

            # Then apply debug_type_error to fix any type errors
            fixed_response, _ = debug_type_error(parsed_response, logger=self.logger)
            if fixed_response:  # Only use the fixed version if it's not empty
                processed_responses.append(fixed_response)
            else:
                # If fixing failed, still use the parsed response
                processed_responses.append(parsed_response)

        # Save all generated samples
        output_dir = Path("output/samples")
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = Path("output/best")
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="01_view_inference",
            logger=self.logger,
        )

        # Initialize and update global best
        global_best_score = (
            context.get_best_score() if hasattr(context, "get_best_score") else None
        )
        global_best_code = (
            context.get_best_code() if hasattr(context, "get_best_code") else None
        )

        # If this is the first global_best_code, initialize it
        if global_best_code is None:
            self.logger.debug(
                f"ViewInference - Initial global_best_code is None: {global_best_code is None}"
            )
            self.logger.debug(
                f"ViewInference - Initial global_best_score: {global_best_score}"
            )
            self.logger.debug(f"ViewInference - Current best_score: {best_score}")
            self.logger.info(
                "ViewInference - Initializing global best with current best"
            )
            global_best_code = best_code
            global_best_score = best_score

        # Save the module-specific best from this step
        module_best_path = output_dir / "01_view_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view inference: {e}")

        # Update context's global best tracking
        context.set_best_code(global_best_code)
        context.set_best_score(global_best_score)

        self.logger.debug(
            f"ViewInference - Stored global best in context with score: {global_best_score}"
        )

        # Add the best sample from current step to context
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code
