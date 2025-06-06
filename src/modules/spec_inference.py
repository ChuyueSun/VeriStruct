"""
Module for inferring requires and ensures clauses in Verus code.
"""

from pathlib import Path
from src.utils.path_utils import samples_dir, best_dir

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import debug_type_error, evaluate_samples, update_checkpoint_best
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
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs if immutable_funcs else []

        # Main instruction for requires/ensures inference
        self.inference_instruction = """You are an expert in Verus (verifier for rust). You have two main tasks:

TASK 1: Add `requires` and `ensures` to public functions where you see "// TODO: add requires and ensures"
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
   - Spec functions (like View) cannot have their own requires/ensures clauses
   - The final code you return MUST compile under Verus; double-check matching braces, parentheses, macro delimiters and remove any remaining "TODO" placeholders
   
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
        examples = []
        try:
            example_path = (
                Path(self.config.get("example_path", "examples")) / "input-requires"
            )
            if example_path.exists():
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = (
                            Path(self.config.get("example_path", "examples"))
                            / "output-requires"
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
        output_dir = samples_dir()
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = best_dir()
        global_dir.mkdir(exist_ok=True, parents=True)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="04_spec_inference",
            logger=self.logger,
        )

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
