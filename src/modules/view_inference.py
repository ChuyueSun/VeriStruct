from typing import List, Dict, Optional, Any
from pathlib import Path
import os

from modules.base import BaseModule
from infer import LLM
from modules.veval import VEval

class ViewInferenceModule(BaseModule):
    """
    Module for View function inference in Verus code.
    
    This module generates a View function that provides a mathematical abstraction
    for the given data structure, which is used in Verus specifications.
    """
    
    def __init__(self, config, logger):
        """
        Initialize the ViewInferenceModule.
        
        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_inference",
            desc="Generate a View function for the data structure's mathematical abstraction"
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
    
    def exec(self, context) -> str:
        """
        Execute the view inference module with the given context.
        
        Args:
            context: Context object containing trial information
            
        Returns:
            Generated code with View function
        """
        self.logger.info("View Inference ...")
        
        # Get the latest trial code
        code = context.trials[-1].code
        
        # Basic instruction
        instruction = """
You are an expert in Verus (verifier for rust). Your task is to generate a View function for the given module. View is the mathematical abstraction for the given data structure. It contains the minimal information to completely represent it. View is used strictly in Verus spec.
    - Add a `View` spec function that provides a mathematical abstraction for types used in the executable code.
    - For `Vec` type variables in the `View`, append "@" to their names.
    - Fill in `/* TODO: part of view */`.
Mathematical types in Verus include:
    - bool
    - int
    - nat
    - Seq<T>
    - Set<T>
    - Map<K, V>

Steps:
    1. Infer the information should be contained in the return type of the `View` function. It could be any of the mathematical types mentioned above or a combination (tuple) of them.
    2. Generate the view function based on the inferred information. Return it as part of the input file.


Format:
```verus

impl<T: Copy> View for RingBuffer<T> {
    type V = // your inferred View return type here that contain the minimal information to represent the class

    closed spec fn view(&self) -> Self::V {
        ... // your implementation here
    }
}
```"""

        # Add important notes
        important_note = """**Important Notes**:
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations."""
        instruction += "\n\n" + important_note
        
        # Add spec knowledge
        spec_knowledge = """**Spec Functions**:
1. No Direct Method Calls:
In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug."""
        instruction += "\n\n" + spec_knowledge
        
        # Add sequence knowledge if needed
        instruction = self.add_seq_knowledge(code, instruction)
        
        # Load examples
        examples = []
        example_path = Path(self.config.example_path) / "input-view"
        if not example_path.exists():
            self.logger.error(f"Example path {example_path} does not exist.")
        else:
            for f in sorted(example_path.iterdir()):
                if f.suffix == ".rs":
                    input_content = f.read_text()
                    answer_path = Path(self.config.example_path) / "output-view" / f.name
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
        
        # Return the best response
        if responses:
            # TODO: More sophisticated selection could be implemented here
            # For now, just return the first response
            new_code = responses[0]
            
            # Add the result to context
            context.add_trial(new_code)
            
            return new_code
        else:
            self.logger.error("View inference failed to generate any responses")
            return code 