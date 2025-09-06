import os
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    save_selection_info,
    update_checkpoint_best,
)
from src.modules.veval import EvalScore, VEval
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, samples_dir


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
            config=config,
            logger=logger,
        )
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

    def _load_examples(self) -> List[Dict[str, str]]:
        """Load example files for view refinement."""
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
        return examples

    def _get_llm_responses(
        self,
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        temperature_boost: float = 0.2,
        retry_attempt: int = 0,
        use_cache: bool = True,
    ) -> List[str]:
        """Get responses from LLM with error handling."""
        try:
            # Add retry marker to instruction to ensure cache miss
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Retry Attempt: {retry_attempt}]"
                use_cache = False  # Disable cache for retries

            # Log the complete query content for debugging
            self.logger.debug("=== LLM Query Content ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(
                f"Temperature: {1.0 + (retry_attempt * temperature_boost)}"
            )
            self.logger.debug(f"Cache Enabled: {use_cache}")
            self.logger.debug("\n=== Instruction ===\n" + instruction)
            self.logger.debug("\n=== Code ===\n" + code)
            if examples:
                self.logger.debug("\n=== Examples ===")
                for i, ex in enumerate(examples):
                    self.logger.debug(f"\nExample {i+1} Query:\n" + ex["query"])
                    self.logger.debug(f"\nExample {i+1} Answer:\n" + ex["answer"])
            self.logger.debug("=====================")

            return self.llm.infer_llm(
                self.config.get("aoai_generation_model", "gpt-4"),
                instruction,
                examples or [],
                code,
                system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                answer_num=3,
                max_tokens=self.config.get("max_token", 8192),
                temp=1.0 + (retry_attempt * temperature_boost),
                use_cache=use_cache,  # Pass cache flag to LLM
            )
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            return []

    def _process_responses(
        self, responses: List[str], original_code: str, context_msg: str = ""
    ) -> List[str]:
        """Process and validate LLM responses."""
        safe_responses = []
        for response in responses:
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            final_response = fixed_response if fixed_response else response
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(f"Generated code passed safety check{context_msg}")
            else:
                self.logger.warning(f"Generated code failed safety check{context_msg}")
        return safe_responses

    def _handle_compilation_retry(
        self,
        code: str,
        original_code: str,
        compile_attempt: int,
        max_compile_attempts: int,
        context,  # Add context parameter
    ) -> List[str]:
        """Handle compilation retry by getting fresh responses."""
        self.logger.info(
            f"Getting fresh responses for compilation attempt {compile_attempt + 1}/{max_compile_attempts}"
        )
        try:
            retry_instruction = build_instruction(
                base_instruction=self.refinement_instruction
                + "\n\nIMPORTANT: Previous attempts resulted in compilation errors. Please ensure the code compiles correctly.",
                add_common=True,
                add_view=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            responses = self._get_llm_responses(
                retry_instruction,
                code,
                temperature_boost=0.3,
                retry_attempt=compile_attempt,
                use_cache=False,  # Explicitly disable cache for compilation retries
            )

            return self._process_responses(
                responses, original_code, context_msg=" in compilation retry"
            )
        except Exception as e:
            self.logger.error(f"Error in compilation retry: {e}")
            return []

    def _save_best_code(
        self,
        best_code: str,
        best_score: EvalScore,
        output_dir: Path,
    ) -> None:
        """Save the best code to a file."""
        module_best_path = output_dir / "02_view_refinement_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best view refinement to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best view refinement: {e}")

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

        # Build instruction and load examples
        instruction = build_instruction(
            base_instruction=self.refinement_instruction,
            add_common=True,
            add_view=True,
            code=code,
            knowledge=context.gen_knowledge(),
        )
        examples = self._load_examples()

        # Initial safety check retries
        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(
                f"View refinement attempt {retry_attempt + 1}/{max_retries}"
            )

            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction,
                code,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0)
            )
            if not responses and retry_attempt == max_retries - 1:
                return code

            safe_responses.extend(self._process_responses(responses, original_code))

            if safe_responses:
                self.logger.info(
                    f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                instruction += f"\n\nIMPORTANT: Previous attempt failed safety checks. Please ensure your View refinement does not modify immutable functions and maintains semantic equivalence. Attempt {retry_attempt + 2}/{max_retries}."

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            safe_responses = [original_code]

        # Setup directories
        output_dir = samples_dir()
        global_dir = best_dir()

        # Compilation retry loop
        max_compile_attempts = 3
        compile_attempt = 0

        while compile_attempt < max_compile_attempts:
            if compile_attempt > 0:
                new_responses = self._handle_compilation_retry(
                    code,
                    original_code,
                    compile_attempt,
                    max_compile_attempts,
                    context,  # Pass context to the method
                )
                if new_responses:
                    safe_responses = new_responses
                else:
                    break

            # Evaluate the samples and get the best one
            best_code, best_score, _ = evaluate_samples(
                samples=safe_responses,
                output_dir=output_dir,
                prefix=f"02_view_refinement_compile_attempt_{compile_attempt + 1}",
                logger=self.logger,
            )

            # Check if there's a compilation error
            if not best_score.compilation_error:
                self.logger.info(
                    f"Found compiling code on attempt {compile_attempt + 1}"
                )
                break

            compile_attempt += 1
            if compile_attempt < max_compile_attempts:
                self.logger.warning(
                    f"Best code has compilation error, will try new responses... (attempt {compile_attempt + 1}/{max_compile_attempts})"
                )
            else:
                self.logger.warning(
                    f"Max compilation attempts reached, falling back to previous best code"
                )
                best_code = context.get_best_code()
                best_score = context.get_best_score()

        # Handle global best tracking
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save results
        self._save_best_code(best_code, best_score, output_dir)

        # Update context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code
