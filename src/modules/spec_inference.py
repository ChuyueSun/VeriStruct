"""
Module for inferring requires and ensures clauses in Verus code.
"""

from typing import List, Dict, Optional, Any
from pathlib import Path
import os

from modules.base import BaseModule
from infer import LLM
from modules.veval import VEval
from modules.utils import evaluate_samples, save_selection_info, update_global_best, debug_type_error

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
            desc="Infer and add requires/ensures clauses to Verus functions"
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs if immutable_funcs else []
        
        # Knowledge specific to requires/ensures clauses
        self.require_ensure_knowledge = """
        REQUIRES AND ENSURES: 
        - `requires` specifies preconditions that must be true before a function executes
        - `ensures` specifies postconditions that will be true after a function executes
        - Both are written after function signatures and before the function body
        
        Example:
        ```
        fn max(a: int, b: int) -> (result: int)
            requires
                true,
            ensures
                result >= a && result >= b,
                (result == a || result == b),
        {
            if a > b { a } else { b }
        }
        ```
        """
    
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
        
        # Build the instruction for the LLM
        instruction = """You are an expert in Verus (verifier for rust). Your task is **Add `requires` and `ensures` to public functions**:
   - Please change the return type of the function if it doesn't have a return type to `-> (retname: rettype)`.
   - Analyze the semantics of the functions and append appropriate `requires` and `ensures` clauses to the method implementations.
   - DO NOT just copy the implementation code. You may use `self.view().XXX` or `self@XXX` in the `ensures` clauses. If `self.view()` is a tuple, you can use `self@.i` to access the i-th element (zero index).
   - DO NOT use `old` without consideration: "only a variable binding is allowed as the argument to old".
   - DO NOT use `match` or `let` in the `ensures` clause.
   - DO NOT add anything to `fn main`.
   - You do not need to add `self.inv()` to the pre- and post-conditions of if `#[verifier::type_invariant]` is used before the `inv` definition.
   - spec functions like View cannot have requires/ensures."""
        
        # Add important notes
        important_note = """**Important Notes**:
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations."""
        instruction += "\n\n" + important_note
        
        # Add knowledge about requires/ensures
        instruction += "\n\n" + self.require_ensure_knowledge
        
        # Add Seq knowledge if Seq is used in the code
        instruction = self.add_seq_knowledge(code, instruction)
        
        # Load examples for spec inference
        examples = []
        try:
            example_path = Path(self.config.get("example_path", "examples")) / "input-requires"
            if not example_path.exists():
                self.logger.error(f"Example path {example_path} does not exist.")
                # Use the latest code as the example
                self.logger.warning("Using latest code as the example")
                examples.append({"query": code, "answer": ""})
            else:
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = Path(self.config.get("example_path", "examples")) / "output-requires" / f.name
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
        output_dir = Path("output/samples")
        output_dir.mkdir(exist_ok=True, parents=True)
        
        # Create a directory for tracking global best samples
        global_dir = Path("output/best")
        global_dir.mkdir(exist_ok=True, parents=True)
        
        # Evaluate processed samples and get the best one
        best_code, best_score, _ = evaluate_samples(
            samples=processed_responses if processed_responses else [code], 
            output_dir=output_dir, 
            prefix="04_spec_inference", 
            logger=self.logger
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
        module_best_path = output_dir / "04_spec_inference_global_best.rs"
        try:
            sample_with_score = f"{global_best_code}\n\n// VEval Score: {global_best_score}"
            module_best_path.write_text(sample_with_score)
            self.logger.info(f"Saved global best spec inference to {module_best_path}")
        except Exception as e:
            self.logger.error(f"Error saving global best: {e}")
        
        # Add the best result to context
        context.add_trial(best_code)
        
        return best_code 