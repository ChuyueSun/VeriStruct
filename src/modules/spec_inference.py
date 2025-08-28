"""
Module for inferring requires and ensures clauses in Verus code.
"""

from pathlib import Path
from src.utils.path_utils import samples_dir, best_dir

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    debug_type_error,
    evaluate_samples,
    update_checkpoint_best,
    get_examples,
    code_change_is_safe,
)
from src.prompts.template import build_instruction
from typing import List, Dict


class SpecInferenceModule(BaseModule):
    """
    Module for inferring requires and ensures clauses for Verus functions.

    This module analyzes the code and adds appropriate preconditions and
    postconditions to functions based on their behavior.
    """

    def __init__(self, config, logger, immutable_funcs=None):
        super().__init__(
            name="spec_inference",
            desc="Infer requires and ensures clauses for functions",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs or []

        # Main instruction for spec inference
        self.inference_instruction = (
            "You are an expert in Verus (verifier for rust). Your task is to:\n\n"
            "1. **Add `requires` and `ensures` to all functions, no matter the function is public or private**:\n"
            "   - Change function signatures without return type to `-> (retname: rettype)`\n"
            "   - Add appropriate `requires` and `ensures` clauses based on function semantics\n"
            "   - For field access in specifications of functions:\n"
            "     * If type T implements View: use `self.view().field` to access fields\n"
            "     * For tuples returned by view(): if `self.view()` returns (A, B), use `self.view().0`, `self.view().1`\n"
            "   - DO NOT use `old(x)` unless x is a simple variable binding\n"
            "   - DO NOT use `match` or `let` in `requires`/`ensures` clauses\n"
            "   - DO NOT modify `fn main()`\n"
            "   - Skip `self.inv()` or `self.well_formed()` in specs when `#[verifier::type_invariant]` is present\n"
            "   - Spec functions (e.g., View) cannot have requires/ensures\n\n"
            "2. **Add `ensures` clauses to trait method implementations**:\n"
            "   - Add appropriate `ensures` clauses based on method semantics\n"
            "   - State conditions that determine the return value\n\n"
            "   - For field access, follow the same rules as above:\n"
            "     * If type implements View: use `self.view().field`\n"
            "     * Otherwise: use direct field access `self.field`\n"
            "   - DO NOT add `requires` clauses to trait implementations (only allowed in trait declarations)\n\n"
            "3. **Implement `spec fn` functions**:\n"
            "   - Write implementation based on function name and context\n"
            "   - Follow field access rules as above for View trait\n"
            "   - You MAY use `match` and `let` inside `spec fn` bodies\n"
            "ADDITIONAL GUIDELINES:\n"
            "   - DO NOT copy implementation code into specifications\n"
            "   - DO NOT modify `fn main()`\n"
            "   - DO NOT delete `// TODO: add proof` or `// TODO: add loop invariant` markers\n"
            "   - DO NOT add loop invariants (leave for proof-generation stage)\n"
            "   - DO NOT add vector length requirements without careful consideration\n"
            "   - DO NOT use AtomicBool::load in requires/ensures clauses\n"
            "   - DO NOT directly compare atomic load with boolean (e.g. atomic.load() == false)\n"
            "   Type System:\n"
            "   - Use `None::<T>` instead of bare `None` for type inference\n"
            "     * CORRECT: `&&& ret == None::<T>`\n"
            "     * INCORRECT: `&&& ret == None`\n\n"
            "   Field Access:\n"
            "   - Check if type implements View before using .view()\n"
            "   - For types without View: use direct field access\n"
            "   - For types with View: use self.view().field\n"
            "   - For tuple views: use self.view().0, self.view().1, etc.\n\n"
            "   Specifications:\n"
            "   - Assert applicable class invariants (self.inv() or #[verifier::type_invariant] or self.well_formed()) in requires/ensures at minimum\n"
            "   - Use old(x) only with simple variable bindings\n"
            "   - NO match/let in requires/ensures (but allowed in spec fn bodies)\n"
            "   - Skip self.inv() when #[verifier::type_invariant] exists\n"
            "   - Spec functions cannot have requires/ensures\n\n"
            "RETURN FORMAT:\n"
            "   - Return the ENTIRE file with your changes integrated into the original code, not just the parts you modified"
        )

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
                use_cache = True
                # use_cache = False  # Disable cache for retries
            
            # Log the complete query content for debugging
            self.logger.debug("=== LLM Query Content ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(f"Temperature: {1.0 + (retry_attempt * temperature_boost)}")
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

    def check_code_safety(self, original_code: str, generated_code: str) -> bool:
        """Check if the generated code is safe to use."""
        # First check if code changes are safe using existing function
        if not code_change_is_safe(
            original_code, generated_code, self.immutable_funcs, self.logger
        ):
            return False
            
        # Check for preservation of TODO markers
        todo_markers = [
            "// TODO: add proof",
            "// TODO: add loop invariant"
        ]
        
        for marker in todo_markers:
            original_count = original_code.count(marker)
            generated_count = generated_code.count(marker)
            
            if original_count > generated_count:
                self.logger.warning(
                    f"Generated code removed {marker} marker(s). "
                    f"Original had {original_count}, generated has {generated_count}."
                )
                return False
                
        return True

    def _process_responses(
        self, 
        responses: List[str], 
        original_code: str,
        context_msg: str = ""
    ) -> List[str]:
        """Process and validate LLM responses."""
        safe_responses = []
        for response in responses:
            # Apply debug_type_error to fix any type errors
            fixed_response, _ = debug_type_error(response, logger=self.logger)
            final_response = fixed_response if fixed_response else response

            # Check if the generated code is safe
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(f"Generated spec code passed safety check{context_msg}")
            else:
                self.logger.warning(f"Generated spec code failed safety check{context_msg}")
        return safe_responses



    def exec(self, context) -> str:
        """
        Execute the spec inference module with the given context.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with inferred requires and ensures clauses
        """
        self.logger.info("Spec Inference (requires/ensures) ...")

        # Get the latest trial code
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(f"Spec inference attempt {retry_attempt + 1}/{max_retries}")

            # Build the complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=self.inference_instruction,
                add_common=True,
                add_requires_ensures=True,  # Include requires/ensures formatting
                add_match=False,  # Include match syntax guidelines
                code=code,
                knowledge="",
                # knowledge=context.gen_knowledge(),
            )
            # Debug log for complete instruction
            self.logger.info("=== Complete Instruction for Debugging ===")
            self.logger.info(instruction)
            self.logger.info("=========================================")
            
            # Load examples for spec inference
            examples = get_examples(self.config, "requires", self.logger)
            
            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction, 
                code, 
                examples, 
                retry_attempt=retry_attempt,
                use_cache=True,
                # use_cache=(retry_attempt == 0)
            )
            if not responses and retry_attempt == max_retries - 1:
                return code

            safe_responses.extend(self._process_responses(responses, original_code))

            if safe_responses:
                self.logger.info(f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts")
                break

            if retry_attempt < max_retries - 1:
                self.inference_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed safety checks. "
                    f"Please ensure your specifications maintain semantic equivalence "
                    f"and do not modify immutable functions. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            return original_code

        # Save all generated samples
        output_dir = samples_dir()
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = best_dir()
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate safe responses and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
            output_dir=output_dir,
            prefix="04_spec_inference",
            logger=self.logger,
        )

        # Final safety check on the best code
        if not self.check_code_safety(original_code, best_code):
            self.logger.warning(
                "Best generated code failed final safety check, falling back to original"
            )
            best_code = original_code

        # Check for compilation errors and attempt repair if needed
        context.add_trial(best_code)  # Add trial to get evaluation results
        latest_trial = context.trials[-1]
        self.logger.info("Latest trial eval:")
        self.logger.info(latest_trial.eval.compilation_error)
        if latest_trial.eval.compilation_error:
            self.logger.info("Detected compilation error, attempting repair...")
            from src.modules.repair_registry import RepairRegistry
            repair_registry = RepairRegistry(self.config, self.logger, self.immutable_funcs)
            repaired_code = repair_registry.repair_compilation_error(context)
            if repaired_code and repaired_code != best_code:
                self.logger.info("Successfully repaired compilation error")
                best_code = repaired_code
                context.add_trial(best_code)  # Add the repaired code as a new trial

        # Get the global best from context
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        # Update global best if current best is better, but don't use it for the current step
        updated_global_best_score, updated_global_best_code = update_checkpoint_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Save the best spec inference from this step to a module-specific file
        module_best_path = output_dir / "04_spec_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best spec inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best spec inference: {e}")

        # Store the updated global best in context
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
