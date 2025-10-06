"""
Baseline module for single-shot specification and proof generation.

This module provides a baseline approach that asks the LLM to generate both
specifications and proofs in a single call, without the multi-stage pipeline.
"""

import os
from pathlib import Path
from typing import List, Dict

from src.modules.base import BaseModule
from src.modules.veval import VEval


class BaselineModule(BaseModule):
    """
    Baseline module that generates specifications and proofs in a single LLM call.
    
    This serves as a baseline comparison against the multi-stage pipeline approach.
    """

    def __init__(self, config, logger, immutable_funcs=None):
        super().__init__(config, logger, immutable_funcs)
        
        # Single comprehensive instruction for both specs and proofs
        self.baseline_instruction = """You are an expert in Verus, a verification framework for Rust. Your task is to complete the given Rust code by adding ALL missing specifications and proofs to make it verify successfully.

TASK: Complete the provided _todo.rs file by adding:
1. **Specifications**: requires/ensures clauses, invariant functions, View implementations
2. **Proofs**: Loop invariants, proof blocks, ghost variables, assertions
3. **Any other verification constructs** needed for successful verification

INSTRUCTIONS:
- Return the COMPLETE file with all TODOs filled in
- Add appropriate requires/ensures clauses to functions
- Implement any missing invariant functions (like `inv`, `well_formed`)  
- Add View implementations for data structures if needed
- Add loop invariants for all loops
- Insert proof blocks where necessary with assertions and lemma calls
- Add ghost variables if they help with verification
- Do NOT change existing function signatures or data structure definitions
- Do NOT modify any non-TODO code unless absolutely necessary for verification
- Focus on making the code verify correctly with Verus

EXAMPLE PROOF CONSTRUCTS:
```rust
// Function specifications
fn my_function(x: i32) -> i32
    requires x > 0,
    ensures result > 0,
{...}

// Loop invariants
while condition {
    invariant property1,
    invariant property2,
    {...}
}

// Proof blocks
proof {
    assert(condition);
    lemma_function(args);
}

// Ghost variables
let ghost old_value = x;

// View implementations
impl View for MyStruct {
    type V = Seq<i32>;
    spec fn view(&self) -> Self::V {...}
}
```

Respond with the complete, corrected Rust code only. Do not include explanations or comments about your changes."""

    def _get_llm_responses(
        self, 
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        retry_attempt: int = 0,
        use_cache: bool = True,
    ) -> List[str]:
        """Get responses from LLM with error handling."""
        try:
            # Add retry marker to instruction to ensure cache miss for retries
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Baseline Retry Attempt: {retry_attempt}]"
                use_cache = False  # Disable cache for retries
            
            # Log the query details
            self.logger.debug("=== Baseline LLM Query ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(f"Temperature: {0.7 + (retry_attempt * 0.1)}")
            self.logger.debug(f"Cache Enabled: {use_cache}")
            self.logger.debug("========================")
                
            return self.llm.infer_llm(
                self.config.get("aoai_generation_model", "gpt-4"),
                instruction,
                examples or [],
                code,
                system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                answer_num=5,  # Generate multiple candidates
                max_tokens=self.config.get("max_token", 16384),
                temp=0.7 + (retry_attempt * 0.1),  # Increase temperature on retries
                use_cache=use_cache,
            )
        except Exception as e:
            self.logger.error(f"Error during baseline LLM inference: {e}")
            return []

    def exec(self, context) -> str:
        """
        Execute the baseline module with a single comprehensive LLM call.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with specifications and proofs
        """
        self.logger.info("=== Baseline Single-Shot Generation ===")

        # Get the initial todo code
        code = context.trials[-1].code
        original_code = code
        
        max_retries = 3
        best_code = code
        best_score = None

        for retry_attempt in range(max_retries):
            self.logger.info(f"Baseline attempt {retry_attempt + 1}/{max_retries}")

            # Get examples if available (but don't require them for baseline)
            examples = []
            try:
                from src.modules.utils import get_examples
                examples = get_examples(self.config, "baseline", self.logger)
            except Exception as e:
                self.logger.debug(f"No baseline examples found: {e}")

            # Get LLM responses
            responses = self._get_llm_responses(
                self.baseline_instruction,
                code,
                examples=examples,
                retry_attempt=retry_attempt,
                use_cache=(retry_attempt == 0)
            )

            if not responses:
                self.logger.warning(f"No responses from LLM on attempt {retry_attempt + 1}")
                continue

            # Process each response and find the best one
            for i, response in enumerate(responses):
                try:
                    # Parse the response to extract code
                    from src.modules.utils import parse_llm_response
                    candidate_code = parse_llm_response(response, self.logger)
                    if not candidate_code.strip():
                        self.logger.warning(f"Empty candidate code from response {i+1}")
                        continue

                    # Check safety if we have immutable functions
                    if not self.check_code_safety(original_code, candidate_code):
                        self.logger.warning(f"Unsafe code change detected in candidate {i+1}, skipping")
                        continue

                    # Evaluate the candidate
                    self.logger.info(f"Evaluating baseline candidate {i+1} from attempt {retry_attempt+1}")
                    veval = VEval(candidate_code, self.logger)
                    score = veval.eval_and_get_score()

                    self.logger.info(f"Candidate {i+1} score: {score}")

                    # Check if this is the best so far
                    if best_score is None or score > best_score:
                        best_score = score
                        best_code = candidate_code
                        self.logger.info(f"New best baseline candidate with score: {score}")

                        # Add trial to context
                        from src.context import Trial
                        trial = Trial(candidate_code, veval, len(context.trials))
                        context.trials.append(trial)

                        # If we found a correct solution, return immediately
                        if score.is_correct():
                            self.logger.info("🎉 Found correct baseline solution!")
                            return candidate_code

                except Exception as e:
                    self.logger.error(f"Error evaluating candidate {i+1}: {e}")
                    continue

            # If we have a good score, we can try to return early
            if best_score and best_score.is_good_code_next_phase():
                self.logger.info(f"Found good baseline code with score {best_score}, stopping early")
                break

        # Log final result
        if best_score:
            self.logger.info(f"Baseline completed with best score: {best_score}")
        else:
            self.logger.warning("Baseline failed to generate any valid candidates")
            best_code = code  # Return original if nothing worked

        return best_code