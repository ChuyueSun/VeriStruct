from typing import List, Dict, Optional, Any
from pathlib import Path
import os

from modules.base import BaseModule
from infer import LLM
from modules.veval import VEval
from modules.utils import evaluate_samples, save_selection_info

class ViewRefinementModule(BaseModule):
    """
    Module for refining View functions in Verus code.
    
    This module improves a View function by ensuring it provides 
    a proper abstraction of the data structure.
    """
    
    def __init__(self, config, logger):
        """
        Initialize the ViewRefinementModule.
        
        Args:
            config: Configuration object
            logger: Logger object
        """
        super().__init__(
            name="view_refinement",
            desc="Refine an existing View function to improve its mathematical abstraction"
        )
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)
    
    def exec(self, context) -> str:
        """
        Execute the view refinement module with the given context.
        
        Args:
            context: Context object containing trial information
            
        Returns:
            Generated code with refined View function
        """
        self.logger.info("View Refinement ...")
        
        # Get the latest trial code
        code = context.trials[-1].code
        
        # Basic instruction
        instruction = """
You are a highly experienced expert in Verus (the verifier for Rust). Your task is to refine the "View" function within the given Verus file. The "View" function is the mathematical abstraction for a data structure, capturing the minimal information needed for its specification in Verus.

Your responsibilities:
  1. Analyze the current "View" function to determine if its tuple (or other structure) adequately represents the module.
  2. Evaluate whether the abstraction can be improved. (Hint: If the tuple is identical to the internal fields, that is likely not an ideal abstraction.)
  3. Modify only the "View" function to improve its abstraction while leaving all other parts of the file unchanged.
  4. Use a flattened tuple.
  5. Return the **entire updated Verus file** with your refined "View" function.

Please provide only the complete Rust code of the refined file with no additional commentary.
"""
        
        # Load examples
        examples = []
        try:
            example_path = Path(self.config.get("example_path", "examples")) / "input-view-refine"
            if not example_path.exists():
                self.logger.error(f"Example path {example_path} does not exist.")
                # Use the latest code as the example
                self.logger.warning("Using latest code as the example")
                examples.append({"query": code, "answer": ""})
            else:
                for f in sorted(example_path.iterdir()):
                    if f.suffix == ".rs":
                        input_content = f.read_text()
                        answer_path = Path(self.config.get("example_path", "examples")) / "output-view-refine" / f.name
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
        
        # Save all generated samples
        output_dir = Path("output/samples")
        output_dir.mkdir(exist_ok=True, parents=True)
        
        # Evaluate samples and get the best one
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code], 
            output_dir=output_dir, 
            prefix="02_view_refinement", 
            logger=self.logger
        )
        
        # Add the best result to context
        context.add_trial(best_code)
        
        return best_code 