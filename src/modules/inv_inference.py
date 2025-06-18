import re
from pathlib import Path

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import debug_type_error, evaluate_samples, update_checkpoint_best, code_change_is_safe
from src.modules.lynette import lynette
from src.prompts.template import build_instruction
from src.utils.path_utils import samples_dir, best_dir


class InvInferenceModule(BaseRepairModule):
    """
    Module for invariant function inference in Verus code.

    This module generates an inv function that captures all
    necessary invariants of a data structure.
    """

    def __init__(self, config, logger):
        """
        Initialize the InvInferenceModule.

        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="inv_inference",
            desc="Generate inv function to capture data structure invariants",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for inv inference
        self.inv_instruction = """You are an expert in Verus (a Rust-based verification framework). Given the following Rust code that defines a data structure with private fields, create a closed spec function: `closed spec fn inv(&self) -> bool`. This function should capture all necessary invariants of the data structure. You are allowed to reference private fields directly (i.e., do not rely on "view" conversions unless absolutely necessary). Do not modify other parts of the code or add explanatory text—just provide the final inv function definition."""

    def replace_at_len_in_type_invariant(self, content: str) -> str:
        """
        Replace all instances of "@.len()" with ".len()" but only within functions
        labeled with #[verifier::type_invariant].

        Args:
            content: The code content to process

        Returns:
            Processed code with corrections
        """
        # Define regex pattern to find type_invariant blocks
        type_invariant_pattern = (
            r"(#\[verifier::type_invariant\][^{]*{((?:[^{}]|(?:\{[^{}]*\}))*)})"
        )

        # Use re.DOTALL to make '.' match newlines as well
        matches = re.finditer(type_invariant_pattern, content, re.DOTALL)

        # Make a copy of the content to modify
        modified_content = content

        # For each match, replace "@.len()" with .len() in the function block
        for match in matches:
            full_match = match.group(
                1
            )  # The entire type_invariant function including the attribute
            function_block = match.group(2)  # Just the function body

            # Replace @.len() with .len() in the function block
            modified_block = re.sub(r"@\.len\(\)", r".len()", function_block)

            # Update the content
            modified_full_match = full_match.replace(function_block, modified_block)
            modified_content = modified_content.replace(full_match, modified_full_match)

        return modified_content

    def exec(self, context) -> str:
        """
        Execute the inv inference module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with inv function
        """
        self.logger.info("Inv Inference ...")

        # Get the latest trial code
        code = context.trials[-1].code

        # Build the complete instruction using the prompt system
        instruction = build_instruction(
            base_instruction=self.inv_instruction,
            add_common=True,
            add_invariant=True,  # Include invariant guidelines
            code=code,
            knowledge=context.gen_knowledge(),
        )

        # Load examples
        examples = []
        try:
            example_path = (
                Path(self.config.get("example_path", "examples")) / "input-inv"
            )
            if example_path.exists():
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = (
                            Path(self.config.get("example_path", "examples"))
                            / "output-inv"
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

        # Save all generated samples (raw responses before processing)
        output_dir = samples_dir()

        # Create a directory for tracking global best samples
        global_dir = best_dir()

        for i, sample in enumerate(responses):
            sample_path = output_dir / f"03_inv_inference_raw_sample_{i+1}.rs"
            try:
                sample_path.write_text(sample)
                self.logger.info(
                    f"Saved inv inference raw sample {i+1} to {sample_path}"
                )
            except Exception as e:
                self.logger.error(f"Error saving raw sample {i+1}: {e}")

        # Process each response to replace @.len() with .len() in type invariants
        processed_responses = []
        original_code = code  # Store original for safety checking
        
        for response in responses:
            processed = self.replace_at_len_in_type_invariant(response)
            # Apply debug_type_error to fix any type errors
            fixed_processed, _ = debug_type_error(processed, logger=self.logger)
            if fixed_processed:  # Only use the fixed version if it's not empty
                processed_responses.append(fixed_processed)
            else:
                processed_responses.append(processed)

        # If we have multiple responses, try merging them using Lynette
        if len(processed_responses) > 1:
            self.logger.info("Attempting to merge multiple invariant candidates using Lynette")
            merged_code = processed_responses[0]  # Start with first candidate
            
            for i in range(1, len(processed_responses)):
                candidate_merge = self.merge_invariants(merged_code, processed_responses[i])
                
                # Check if the merge is safe
                if self.check_code_safety(merged_code, candidate_merge):
                    merged_code = candidate_merge
                    self.logger.info(f"Successfully merged candidate {i+1}")
                else:
                    self.logger.warning(f"Merge with candidate {i+1} deemed unsafe, keeping previous version")
            
            # Add the merged result as an additional candidate
            processed_responses.append(merged_code)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="03_inv_inference_processed",
            logger=self.logger,
        )

        # Final safety check on the best code
        if not self.check_code_safety(original_code, best_code):
            self.logger.warning("Best generated code failed safety check, falling back to original")
            best_code = original_code

        # Get the global best from context
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        # Update global best if current best is better, but don't use it for the current step
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best inv inference from this step to a module-specific file
        module_best_path = output_dir / "03_inv_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best inv inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best inv inference: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context, regardless of global best
        context.add_trial(best_code)  # Always use the best sample from this step

        # If the global best is significantly better than what we just generated,
        # consider returning the global best instead
        if (
            global_best_score
            and best_score
            and global_best_score.is_correct()
            and not best_score.is_correct()
        ):
            self.logger.info(
                "Using global best code as it is correct while current best is not"
            )
            return global_best_code

        return best_code

    def merge_invariants(self, code1: str, code2: str) -> str:
        """
        Merge invariants from two code versions using Lynette.
        
        Args:
            code1: First code version
            code2: Second code version
            
        Returns:
            Merged code or code2 if merging fails
        """
        try:
            import tempfile
            
            with tempfile.NamedTemporaryFile(mode="w", suffix=".rs", delete=False) as f1, \
                 tempfile.NamedTemporaryFile(mode="w", suffix=".rs", delete=False) as f2:
                
                f1.write(code1)
                f1.flush()
                f2.write(code2)
                f2.flush()
                
                result = lynette.code_merge_invariant(f1.name, f2.name)
                
                # Clean up temp files
                import os
                os.unlink(f1.name)
                os.unlink(f2.name)
                
                if result.returncode == 0:
                    self.logger.info("Successfully merged invariants using Lynette")
                    return result.stdout.strip()
                else:
                    self.logger.warning(f"Lynette invariant merge failed: {result.stderr}")
                    return code2
                    
        except Exception as e:
            self.logger.error(f"Error during invariant merging: {e}")
            return code2

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
