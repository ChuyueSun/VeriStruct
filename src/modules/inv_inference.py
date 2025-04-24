import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from infer import LLM
from modules.base import BaseModule
from modules.utils import (
    debug_type_error,
    evaluate_samples,
    save_selection_info,
    update_global_best,
    write_candidate_code,
)
from modules.veval import VEval


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
            desc="Generate inv function to capture data structure invariants",
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)

    def add_seq_knowledge(self, code: str, instruction: str) -> str:
        """
        Add knowledge about Seq operations if needed for the given code.

        Args:
            code: The Verus code
            instruction: The current instruction

        Returns:
            Updated instruction with sequence knowledge if needed
        """
        if "Seq" in code:
            seq_knowledge = """**Seq Knowledge**:
Seq<T> is a mathematical sequence type used in specifications:
- Building: Seq::empty(), seq![x, y, z], Seq::singleton(x)
- Length: s.len()
- Indexing: s[i] (0-based)
- Subrange: s.subrange(lo, hi) gives elements from index lo (inclusive) to hi (exclusive)
- Concatenation: s1 + s2
- Update: s.update(i, v) returns a new sequence with index i updated to value v
- Contains: s.contains(v) checks if v is in the sequence
- Push/pop: s.push(v), s.pop() (returns new sequence, doesn't modify original)
You can use forall or exists for properties over sequences."""
            instruction += "\n\n" + seq_knowledge
        return instruction

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

        # Basic instruction
        instruction = """You are an expert in Verus (a Rust-based verification framework). Given the following Rust code that defines a data structure with private fields, create a closed spec function: `closed spec fn inv(&self) -> bool`. This function should capture all necessary invariants of the data structure. You are allowed to reference private fields directly (i.e., do not rely on "view" conversions unless absolutely necessary). Do not modify other parts of the code or add explanatory text—just provide the final inv function definition."""

        # Add important notes
        important_note = """**Important Notes**:
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations."""
        instruction += "\n" + important_note

        # Add spec knowledge
        spec_knowledge = """**Spec Functions**:
1. No Direct Method Calls:
In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug."""
        instruction += "\n" + spec_knowledge

        # Add sequence knowledge if needed
        instruction = self.add_seq_knowledge(code, instruction)

        # Load examples
        examples = []
        try:
            example_path = (
                Path(self.config.get("example_path", "examples")) / "input-inv"
            )
            if not example_path.exists():
                self.logger.error(f"Example path {example_path} does not exist.")
                # Use the latest code as the example
                self.logger.warning("Using latest code as the example")
                examples.append({"query": code, "answer": ""})
            else:
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
        except Exception as e:
            self.logger.error(f"Error loading examples: {e}")
            # If we failed to create examples, at least create an empty one
            if not examples:
                examples.append({"query": code, "answer": ""})

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
        output_dir = Path("output/samples")
        output_dir.mkdir(exist_ok=True, parents=True)

        # Create a directory for tracking global best samples
        global_dir = Path("output/best")
        global_dir.mkdir(exist_ok=True, parents=True)

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
        for response in responses:
            processed = self.replace_at_len_in_type_invariant(response)
            # Apply debug_type_error to fix any type errors
            fixed_processed, _ = debug_type_error(processed, logger=self.logger)
            if fixed_processed:  # Only use the fixed version if it's not empty
                processed_responses.append(fixed_processed)
            else:
                processed_responses.append(processed)

        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code],
            output_dir=output_dir,
            prefix="03_inv_inference_processed",
            logger=self.logger,
        )

        # Get the global best from context
        global_best_score = context.get_best_score()
        global_best_code = context.get_best_code()

        # Update global best if current best is better
        global_best_score, global_best_code = update_global_best(
            best_code, global_best_score, global_best_code, global_dir, self.logger
        )

        # Store the updated global best in context
        context.set_best_score(global_best_score)
        context.set_best_code(global_best_code)

        # Also write to a module-specific best file
        module_best_path = output_dir / "03_inv_inference_global_best.rs"
        try:
            sample_with_score = (
                f"{global_best_code}\n\n// VEval Score: {global_best_score}"
            )
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved global best inv inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving global best: {e}")

        # Add the best result to context
        context.add_trial(best_code)

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
