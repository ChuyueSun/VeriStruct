import re
from pathlib import Path
from typing import Dict, List

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.lynette import lynette
from src.modules.utils import (
    code_change_is_safe,
    debug_type_error,
    evaluate_samples,
    get_examples,
    update_checkpoint_best,
)
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, prompt_dir, samples_dir


class InvInferenceModule(BaseModule):
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
            desc="Generate invariant functions for the data structure",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)

        # Main instruction for inv inference
        self.inv_instruction = """You are an expert in Verus (a Rust-based verification framework). Given the following Rust code that defines a data structure with private fields, implement the invariant functions that are already declared in the code. Common names for these functions are `well_formed`, `inv`, or `invariant`. You are allowed to reference private fields directly (i.e., do not rely on "view" conversions unless absolutely necessary).

IMPORTANT:
- ONLY implement invariant functions that already exist in the code - do not create new ones.
- Look for functions named `well_formed`, `inv`, `invariant`, `inv`, or similar that are marked with TODO or are empty.
- Do NOT rename existing functions or create new `spec fn inv` functions unless explicitly requested.
- When `struct_with_invariants` is present in the input file, use library knowledge to construct the correct invariant. Use `invariant on field with` to construct the invariants for the target class.
- **CRITICAL - Choosing between implication (==>) and biconditional (===):**
  * Use IMPLICATION (==>) when expressing "elements/values that exist in a collection must satisfy a property"
    - Pattern: "forall |x| collection.contains(x) ==> property(x)" means "if x is in collection, then property holds"
    - This does NOT claim that all values satisfying the property must be in the collection
  * Use BICONDITIONAL (===) ONLY when two predicates are logically equivalent in both directions
    - Pattern: "predicate_A(x) === predicate_B(x)" means both predicates are always true or false together
    - Use for equivalence of two different representations of the same fact
  * Default to implication (==>) for structural invariants on sparse/selective data structures (trees, maps, filtered collections)
  * Most invariants constrain "what is present" not "what must be present" - use implication for these
- Return the ENTIRE file with your changes integrated into the original code, not just the inv function definition.
- Do not modify other parts of the code.
- Do not add explanatory text.
- Do NOT fill in any proofs or non-inv specifications - leave all TODOs and proof obligations untouched.
- Focus ONLY on implementing existing invariant functions - do not attempt to complete any other specifications or proofs.
- If you find multiple invariant functions to implement (e.g., both `well_formed` and `inv`), implement all of them while preserving their original names.

CRITICAL: Quantifier Syntax
- `forall` and `exists` are KEYWORDS, not methods
- CORRECT: forall |x| collection.contains(x) ==> condition(x)
- WRONG: collection.forall(|x| condition(x))
- Sets/Maps have .contains(), .dom(), but NOT .forall() or .exists()
- Use the keyword syntax: forall |var| predicate, not method call syntax"""

    def _get_llm_responses(
        self,
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        temperature_boost: float = 0.2,
        retry_attempt: int = 0,
        use_cache: bool = True,
        context=None,
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

            engine = self.config.get("aoai_generation_model", "gpt-4")
            if context is not None:
                result = context.infer_llm_with_tracking(
                    engine=engine,
                    instruction=instruction,
                    exemplars=examples or [],
                    query=code,
                    system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                    answer_num=3,
                    max_tokens=self.config.get("max_token", 8192),
                    temp=1.0 + (retry_attempt * temperature_boost),
                    use_cache=use_cache,  # Pass cache flag to LLM
                    stage="inv_inference",
                    module="inv_inference",
                )
                if isinstance(result, tuple):
                    result = result[0]
                return result
            else:
                return self.llm.infer_llm(
                    engine,
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
            processed = self.replace_at_len_in_type_invariant(response)
            # Apply debug_type_error to fix any type errors
            fixed_processed, _ = debug_type_error(processed, logger=self.logger)
            temp_response = fixed_processed if fixed_processed else processed

            # Apply regex-based syntax fixes
            from src.modules.repair_regex import fix_common_syntax_errors

            final_response, was_changed = fix_common_syntax_errors(temp_response, self.logger)
            if was_changed:
                self.logger.info("Applied regex syntax fixes to invariant inference response")

            # Check if the generated code is safe
            if self.check_code_safety(original_code, final_response):
                safe_responses.append(final_response)
                self.logger.info(f"Generated invariant code passed safety check{context_msg}")
            else:
                self.logger.warning(f"Generated invariant code failed safety check{context_msg}")
        return safe_responses

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
        original_code = code  # Store original for safety checking

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info(f"Inv inference attempt {retry_attempt + 1}/{max_retries}")

            # Build the complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=self.inv_instruction,
                add_common=True,
                add_invariant=True,  # Include invariant guidelines
                add_match=True,
                code=code,
                knowledge=context.gen_knowledge(),
            )

            # Load examples showing completed invariants (answer-only format)
            # This reduces redundancy - we only show the pattern, not the before/after
            raw_examples = get_examples(self.config, "inv", self.logger, max_examples=8)

            # Convert to answer-only format: use 'answer' as both query and answer
            # This shows the LLM the correct pattern without redundant TODO version
            examples = []
            for i, ex in enumerate(raw_examples):
                if ex.get("answer"):
                    examples.append(
                        {
                            "query": f"Example {i+1}: Pattern for implementing invariant functions",
                            "answer": ex["answer"],
                        }
                    )

            self.logger.info(
                f"Using {len(examples)} answer-only examples from output-inv (reduced redundancy)"
            )

            # Save prompt for debugging
            prompt_path = prompt_dir()
            prompt_file = prompt_path / f"inv_inference_{retry_attempt + 1}.txt"
            prompt_file.write_text(instruction)
            self.logger.info(f"Saved inv inference prompt to {prompt_file}")

            # Use cache only for first attempt
            responses = self._get_llm_responses(
                instruction,
                code,
                examples=examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                context=context,
                # use_cache=(retry_attempt == 0)
            )
            if not responses and retry_attempt == max_retries - 1:
                return code

            # Save raw samples
            output_dir = samples_dir()
            output_dir.mkdir(exist_ok=True, parents=True)

            for i, sample in enumerate(responses):
                sample_path = (
                    output_dir / f"03_inv_inference_raw_sample_{i+1}_attempt_{retry_attempt+1}.rs"
                )
                try:
                    sample_path.write_text(sample)
                    self.logger.info(
                        f"Saved inv inference raw sample {i+1} from attempt {retry_attempt+1} to {sample_path}"
                    )
                except Exception as e:
                    self.logger.error(f"Error saving raw sample {i+1}: {e}")

            safe_responses.extend(self._process_responses(responses, original_code))

            if safe_responses:
                self.logger.info(
                    f"Found {len(safe_responses)} safe responses after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                self.inv_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed safety checks. "
                    f"Please ensure your invariant implementation maintains semantic equivalence "
                    f"and does not modify existing code. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning("No safe responses found after all retries, using original code")
            return original_code

        # Create a directory for tracking global best samples
        global_dir = best_dir()

        # Evaluate safe responses and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=safe_responses,
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

        # Add the best sample from current step to context
        context.add_trial(best_code)

        return best_code
