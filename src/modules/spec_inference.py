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


class SpecInferenceModule(BaseModule):
    """
    Module for inferring requires and ensures clauses for Verus functions.

    This module analyzes the code and adds appropriate preconditions and
    postconditions to functions based on their behavior.
    """

    def __init__(self, config, logger, immutable_funcs=None):
        """
        Initialize the SpecInferenceModule.

        Args:
            config: Configuration object
            logger: Logger object
            immutable_funcs: List of function names that should not be modified
        """
        super().__init__(
            name="spec_inference",
            desc="Infer and add requires/ensures clauses to Verus functions",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )
        self.llm = LLM(config, logger)

        # Main instruction for requires/ensures inference
        self.inference_instruction = """You are an expert in Verus (verifier for rust). You have two main tasks:

TASK 1: Add `requires` and `ensures` to public functions where you see "// TODO: add requires and ensures"
   - If a type has `#[verifier::type_invariant]`, DO NOT assert invariants explicitly in requires/ensures - the type invariant is automatically maintained
   - If NO `#[verifier::type_invariant]` exists, consider asserting class invariants (`well-formed`, `invariants`, `inv` etc.) in pre/post-conditions
   - Analyze the semantics of functions and add appropriate preconditions and postconditions
   - Change function signatures to `-> (retname: rettype)` format when adding return value specifications
   - Use precise, mathematical specifications that capture the function's behavior

TASK 2: Fill in `spec fn` implementations where you see "TODO: add specification"
   - Implement the specification function based on the context and function name

IMPORTANT GUIDELINES:
   - DO NOT just copy the implementation code in specifications
   - You may use `self.view().XXX` or `self@XXX` in `ensures` clauses
   - If `self.view()` is a tuple, you can use `self@.i` to access the i-th element (zero-indexed)
   - DO NOT use `old` without consideration: "only a variable binding is allowed as the argument to old"
   - DO NOT use `match` or `let` in the `ensures` clause or `requires` clause, but you can use `match` within `spec fn` bodies
   - DO NOT modify anything in `fn main()`
   - DO NOT add `self.inv()` to pre/post-conditions if `#[verifier::type_invariant]` is used
   - DO NOT delete any `// TODO: add proof` or `// TODO: add invariants` comment; DO NOT add loop invariants yet; leave it intact for the proof-generation stage
   - DO NOT add vector length requirements like "requires old(v).len() < u64::MAX - 1 as usize" without careful consideration
   - Spec functions (like View) cannot have their own requires/ensures clauses
   - The final code you return MUST compile under Verus; double-check matching braces, parentheses, macro delimiters and remove any remaining "TODO" placeholders
   - Do not use AtomicBool::load in requires/ensures clauses
   - At the very minimum, simply assert class invariants in requires/ensures clauses

   RETURN FORMAT:
   - Return the ENTIRE file with your changes integrated into the original code, not just the parts you modified"""

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
            # Run inference with increasing temperature on retries
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
                    return code  # Fallback to original code on last attempt
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
                    self.logger.info("Generated spec code passed safety check")
                else:
                    self.logger.warning(
                        "Generated spec code failed safety check, will retry"
                    )

            # If we have safe responses, break out of retry loop
            if safe_responses:
                self.logger.info(f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts")
                break

            # If this is not the last attempt, modify instruction for retry
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

        # Also write to a module-specific best file
        module_best_path = output_dir / "04_spec_inference_global_best.rs"
        try:
            sample_with_score = f"{best_code}\n\n// VEval Score: {best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved best spec inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best spec inference: {e}")

        # Store the updated global best in context, but use the current best sample for the next step
        context.set_best_score(updated_global_best_score)
        context.set_best_code(updated_global_best_code)

        # Add the best sample from current step to context, regardless of global best
        context.add_trial(best_code)  # Always use the best sample from this step

        return best_code
