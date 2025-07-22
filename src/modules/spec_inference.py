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
            "1. **Add `requires` and `ensures` to public functions**:\n"
            "   - Please change the return type of the function if it doesn't have a return type to `-> (retname: rettype)`.\n"
            "   - Analyze the semantics of the functions and append appropriate `requires` and `ensures` clauses to the method implementations.\n"
            "   - DO NOT just copy the implementation code. You may use `self.view().XXX` or `self@XXX` in the `ensures` clauses. If `self.view()` is a tuple, you can use `self@.i` to access the i-th element (zero index).\n"
            "   - DO NOT use `old` without consideration: \"only a variable binding is allowed as the argument to old\".\n"
            "   - DO NOT use `match` or `let` in the `ensures` clause.\n"
            "   - DO NOT add anything to `fn main`.\n"
            "   - You do not need to add `self.inv()` to the pre- and post-conditions if `#[verifier::type_invariant]` is used before the `inv` definition.\n"
            "   - spec functions like View cannot have requires/ensures.\n\n"
            "2. **Add `ensures` clauses for trait methods**:\n"
            "   - Analyze the semantics of the functions and append appropriate `ensures` clauses to the trait method implementations.\n"
            "   - DO NOT just copy the implementation code. You may use `self.view().XXX` in the `ensures` clauses.\n"
            "   - DO NOT add the `requires` clause to the trait method implementations. This is not allowed: \"trait method implementation cannot declare requires clauses; these can only be inherited from the trait declaration\"\n\n"
            "3. **Fill in `spec fn` implementations**:\n"
            "   - Implement the specification function based on the context and function name\n"
            "   - State what implies the return value\n\n"
            "IMPORTANT GUIDELINES:\n"
            "   - DO NOT just copy the implementation code in specifications\n"
            "   - You may use `self.view().XXX` or `self@XXX` in `ensures` clauses\n"
            "   - If `self.view()` is a tuple, you can use `self@.i` to access the i-th element (zero-indexed)\n"
            "   - DO NOT use `old` without consideration: \"only a variable binding is allowed as the argument to old\"\n"
            "   - DO NOT use `match` or `let` in the `ensures` clause or `requires` clause, but you can use `match` within `spec fn` bodies\n"
            "   - DO NOT modify anything in `fn main()`\n"
            "   - DO NOT add `self.inv()` to pre/post-conditions if `#[verifier::type_invariant]` is used\n"
            "   - DO NOT delete any `// TODO: add proof` or `// TODO: add invariants` comment; "
            "   - DO NOT add loop invariants yet; leave it intact for the proof-generation stage\n"
            "   - DO NOT add vector length requirements like \"requires old(v).len() < u64::MAX - 1 as usize\" without careful consideration\n"
            "   - Spec functions (like View) cannot have their own requires/ensures clauses\n"
            "   - Do not use AtomicBool::load in requires/ensures clauses\n"
            "   - At the very minimum, simply assert class invariants in requires/ensures clauses\n"
            "   - ALWAYS use `None::<T>` instead of bare `None` to help type inference. For example:\n"
            "     * CORRECT: `&&& ret == None::<T>`\n"
            "     * INCORRECT: `&&& ret == None`\n\n"
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
                use_cache = False  # Disable cache for retries
            
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
                add_match=True,  # Include match syntax guidelines
                code=code,
                knowledge=context.gen_knowledge(),
            )

            # Load examples for spec inference
            examples = get_examples(self.config, "requires", self.logger)
            
            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction, 
                code, 
                examples, 
                retry_attempt=retry_attempt,
                use_cache=(retry_attempt == 0)
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
