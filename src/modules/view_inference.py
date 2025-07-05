import re
from pathlib import Path

from src.context import Context
from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    update_checkpoint_best,
    get_examples,
    code_change_is_safe,
    parse_llm_response,
)
from src.prompts.template import build_instruction
from src.utils.path_utils import samples_dir, best_dir


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
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for View inference
        self.view_instruction = """
You are an expert in Verus (verifier for rust). Your task is to generate a View function for the given module. View is the mathematical abstraction for the given data structure. It contains the minimal information to completely represent it. View is used strictly in Verus spec.
    - Add a `View` spec function that provides a mathematical abstraction for types used in the executable code.
    - For `Vec` type variables in the `View`, append "@" to their names.
    - Fill in only `/* TODO: part of view */`.
    - Do NOT use `reveal` keyword in the View implementation.
Mathematical types in Verus include:
    - bool
    - int
    - nat
    - Seq<T>
    - Set<T>
    - Map<K, V>

Steps:
    1. Infer the information should be contained in the return type of the `View` function. It could be any of the mathematical types mentioned above or a combination (tuple) of them.
    2. Generate the view function based on the inferred information.
    3. Return the ENTIRE file with your changes, not just the View implementation.


Format for the View implementation:
```verus
impl<T: Copy> View for RingBuffer<T> {
    type V = // your inferred View return type here that contain the minimal information to represent the class

    closed spec fn view(&self) -> Self::V {
        ... // your implementation here
    }
}
```

IMPORTANT: Return the complete file with your changes integrated into the original code."""

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

    def exec(self, context: Context) -> str:
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
        original_code = code  # Store original for safety checking

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.view_instruction,
            add_common=True,
            add_view=True,  # Include View guidelines
            code=code,
            knowledge=context.gen_knowledge(),
        )

        # Load examples
        examples = get_examples(self.config, "view", self.logger)
        # Retry mechanism for safety checks
        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"View inference attempt {retry_attempt + 1}/{max_retries}"
            )

            # Run inference
            try:
                responses = self.llm.infer_llm(
                    self.config.get("aoai_generation_model", "gpt-4"),
                    instruction,
                    examples,
                    code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 20000),
                    temp=1.0 + (retry_attempt * 0.2),  # Increase temperature on retries
                )
            except Exception as e:
                self.logger.error(f"Error during LLM inference: {e}")
                if retry_attempt == max_retries - 1:
                    # Last attempt failed, return original code
                    return code
                continue

            # Parse and process responses
            processed_responses = []
            for response in responses:
                # First parse the response to extract the View implementation
                final_response = parsed_response = parse_llm_response(response)

                # Then apply debug_type_error to fix any type errors
                fixed_response, _ = debug_type_error(
                    parsed_response, logger=self.logger
                )
                final_response = fixed_response if fixed_response else parsed_response

                # Check if the generated code is safe
                if self.check_code_safety(original_code, final_response):
                    processed_responses.append(final_response)
                    safe_responses.append(final_response)
                    self.logger.info("Generated view code passed safety check")
                else:
                    self.logger.warning(
                        "Generated view code failed safety check, will retry"
                    )

            # If we have safe responses, break out of retry loop
            if safe_responses:
                self.logger.info(
                    f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts"
                )
                break

            # If this is not the last attempt, modify instruction for retry
            if retry_attempt < max_retries - 1:
                instruction += f"\n\nIMPORTANT: Previous attempt failed safety checks. Please ensure your View implementation does not modify immutable functions and maintains semantic equivalence. Attempt {retry_attempt + 2}/{max_retries}."

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            safe_responses = [original_code]

        # Save all generated samples
        output_dir = samples_dir()
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = best_dir()
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="01_view_inference",
            logger=self.logger,
        )

        # Initialize and update global best
        checkpoint_best_score = (
            context.get_best_score() if hasattr(context, "get_best_score") else None
        )
        checkpoint_best_code = (
            context.get_best_code() if hasattr(context, "get_best_code") else None
        )

        # If this is the first checkpoint_best_code, initialize it
        if checkpoint_best_code is None:
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_code is None: {checkpoint_best_code is None}"
            )
            self.logger.debug(
                f"ViewInference - Initial checkpoint_best_score: {checkpoint_best_score}"
            )
            self.logger.debug(f"ViewInference - Current best_score: {best_score}")
            self.logger.info(
                "ViewInference - Initializing checkpoint best with current best"
            )
            checkpoint_best_code = best_code
            checkpoint_best_score = best_score

        # Save the module-specific best from this step
        module_best_path = output_dir / "01_view_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view inference: {e}")

        # Update context's global best tracking
        context.set_best_code(checkpoint_best_code)
        context.set_best_score(checkpoint_best_score)

        self.logger.debug(
            f"ViewInference - Stored checkpoint best in context with score: {checkpoint_best_score}"
        )

        # Add the best sample from current step to context
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code
