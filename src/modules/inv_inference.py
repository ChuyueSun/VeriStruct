from typing import List, Dict, Optional, Any
from pathlib import Path
import os
import re

from modules.base import BaseModule
from infer import LLM
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
            desc="Generate inv function to capture data structure invariants"
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
        type_invariant_pattern = r'(#\[verifier::type_invariant\][^{]*{((?:[^{}]|(?:\{[^{}]*\}))*)})'
        
        # Use re.DOTALL to make '.' match newlines as well
        matches = re.finditer(type_invariant_pattern, content, re.DOTALL)
        
        # Make a copy of the content to modify
        modified_content = content
        
        # For each match, replace "@.len()" with .len() in the function block
        for match in matches:
            full_match = match.group(1)  # The entire type_invariant function including the attribute
            function_block = match.group(2)  # Just the function body
            
            # Replace @.len() with .len() in the function block
            modified_block = re.sub(r'@\.len\(\)', r'.len()', function_block)
            
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
        example_path = Path(self.config.example_path) / "input-inv"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = Path(self.config.example_path) / "output-inv" / f.name
                    answer = answer_path.read_text() if answer_path.exists() else ""
                    examples.append({"query": input_content, "answer": answer})
        
        # Run inference
        responses = self.llm.infer_llm(
            self.config.aoai_generation_model,
            instruction,
            examples,
            code,
            system_info="You are a helpful AI assistant specialized in Verus formal verification.",
            answer_num=3,
            max_tokens=self.config.max_token,
            temp=1.0,
        )
        
        # Process each response to replace @.len() with .len() in type invariants
        processed_responses = []
        for response in responses:
            processed = self.replace_at_len_in_type_invariant(response)
            processed_responses.append(processed)
            
        # Return the best response
        if processed_responses:
            # TODO: More sophisticated selection could be implemented here
            # For now, just return the first response
            new_code = processed_responses[0]
            
            # Add the result to context
            context.add_trial(new_code)
            
            return new_code
        else:
            self.logger.error("Inv inference failed to generate any responses")
            return code 