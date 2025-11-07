"""
Base class for Repair modules in VeriStruct.
"""

import logging
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import code_change_is_safe
from src.modules.veval import VerusError, VerusErrorType, VEval
from src.prompts.template import fill_template


class BaseRepairModule(BaseModule):
    """
    Base class for all repair modules.
    Repair modules focus on fixing specific types of Verus errors.
    """

    def __init__(
        self,
        name: str,
        desc: str,
        config: Dict[str, Any],
        logger: logging.Logger,
        immutable_funcs: Optional[List[str]] = None,
    ):
        """
        Initialize the BaseRepairModule.

        Args:
            name: Name of the repair module
            desc: Description of the repair module
            config: Configuration dictionary
            logger: Logger instance
            immutable_funcs: List of function names that should not be modified
        """
        super().__init__(
            name=name,
            desc=desc,
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )
        self.llm = LLM(config, logger)

        # Common knowledge strings can be initialized here if needed
        self.proof_block_info = """The proof block looks like this:
```
proof {
    // your proof code here
    // assert(...)
    // LEMMA_FUNCTION(...)
    // ...
} // Added by AI
```
Note, please add the assertion directly for the `proof fn` function and DO NOT use proof block.
You can only use the proof block for the `fn` and `pub fn` functions.

The ghost variable looks like this:
```
let ghost ...; // Added by AI
```

Note, please DO NOT modify all other proof blocks that are not related to the error. Just leave them as they are."""
        self.seq_knowledge = """**Seq Knowledge**:
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
        self.general_knowledge = f"""IMPORTANT:
1. Don't change the anything in immutable function(s): {', '.join(self.immutable_funcs)}. Instead, consider adjusting the preconditions or postconditions of other functions or methods.
2. Don't delete existing non-buggy `#[trigger]`, `use` statements, main function.
"""

    def add_seq_knowledge(self, code: str, instruction: str) -> str:
        """Check whether the code contains the usage of Seq/Vec and add the Seq knowledge to the instruction."""
        _possible_usage = ["Seq", "Vec", "array", "nums"]
        for usage in _possible_usage:
            if usage in code:
                instruction += "\n\n" + self.seq_knowledge
                break
        return instruction

    def get_one_failure(self, failures: List[VerusError]) -> Optional[VerusError]:
        """Selects a single failure to focus on from a list of Verus errors."""
        # Simple strategy: prioritize based on error type or just take the first one
        # More sophisticated prioritization can be added later.
        if not failures:
            return None
        # Prioritize specific errors if needed, otherwise return the first
        # Example prioritization (can be expanded):
        priority = [
            VerusErrorType.MismatchedType,
            VerusErrorType.PreCondFail,
            VerusErrorType.PostCondFail,
            VerusErrorType.InvFailEnd,
            VerusErrorType.InvFailFront,
            VerusErrorType.AssertFail,
        ]
        for err_type in priority:
            for failure in failures:
                if failure.error == err_type:
                    self.logger.info(f"Prioritizing failure: {failure.error.name}")
                    return failure
        # Fallback to the first error
        self.logger.info(f"Selecting first failure: {failures[0].error.name}")
        return failures[0]

    def evaluate_repair_candidates(
        self, original_code: str, candidates: List[str], output_dir, prefix: str
    ) -> str:
        """
        Evaluate repair candidates with safety checking.

        Args:
            original_code: Original code before repair
            candidates: List of candidate repairs
            output_dir: Directory for saving evaluation results
            prefix: Prefix for output files

        Returns:
            Best safe candidate code
        """
        from src.modules.utils import evaluate_samples

        # Filter candidates by safety first
        safe_candidates = []
        for candidate in candidates:
            if self.check_code_safety(original_code, candidate):
                safe_candidates.append(candidate)
                self.logger.info("Repair candidate passed safety check")
            else:
                self.logger.warning(
                    "Repair candidate failed safety check, excluding from evaluation"
                )

        # If no candidates are safe, fall back to original
        if not safe_candidates:
            self.logger.warning("No safe repair candidates found, returning original code")
            return original_code

        # Evaluate safe candidates and return the best one
        best_code, _, _ = evaluate_samples(
            samples=safe_candidates,
            output_dir=output_dir,
            prefix=prefix,
            logger=self.logger,
        )

        return best_code

    def _get_llm_responses(
        self,
        instruction: str,
        query: str,
        examples: List[Dict[str, str]] = None,
        temperature_boost: float = 0.2,
        retry_attempt: int = 0,
        use_cache: bool = True,
        context=None,  # Optional context for appending knowledge
        timeout: int = None,  # Timeout in seconds
    ) -> List[str]:
        """
        Get responses from LLM with error handling and timeout protection.

        Args:
            instruction: The instruction for the LLM
            query: The query/code to process
            examples: Optional list of example dictionaries with 'query' and 'answer' keys
            temperature_boost: Amount to increase temperature per retry
            retry_attempt: Current retry attempt number
            use_cache: Whether to use response caching
            context: Optional context object for appending knowledge
            timeout: Maximum time in seconds to wait for LLM response (default: 60s)

        Returns:
            List of response strings from the LLM, or empty list if timeout
        """
        import time

        # Default timeout for repair operations
        if timeout is None:
            timeout = self.config.get("repair_llm_timeout", 60)

        llm_start_time = time.time()

        try:
            # Add retry marker to instruction to ensure cache miss
            if retry_attempt > 0:
                instruction = f"{instruction}\n[Retry Attempt: {retry_attempt}]"
                use_cache = False  # Disable cache for retries

            # Append context knowledge to query if available
            final_query = query
            if context and hasattr(context, "gen_knowledge"):
                final_query += "\n\nAdditional Context:\n" + context.gen_knowledge()

            # Log the complete query content for debugging
            self.logger.debug("=== LLM Query Content ===")
            self.logger.debug(f"Retry Attempt: {retry_attempt}")
            self.logger.debug(f"Temperature: {1.0 + (retry_attempt * temperature_boost)}")
            self.logger.debug(f"Cache Enabled: {use_cache}")
            self.logger.debug("\n=== Instruction ===\n" + instruction)
            self.logger.debug("\n=== Query ===\n" + final_query)
            if examples:
                self.logger.debug("\n=== Examples ===")
                for i, ex in enumerate(examples):
                    self.logger.debug(f"\nExample {i+1} Query:\n" + ex["query"])
                    self.logger.debug(f"\nExample {i+1} Answer:\n" + ex["answer"])
            self.logger.debug("=====================")

            engine = self.config.get("aoai_debug_model", "gpt-4")
            temp = 1.0 + (retry_attempt * temperature_boost)

            # Use tracking wrapper if context is available, otherwise use direct call
            if context is not None and hasattr(context, "infer_llm_with_tracking"):
                result = context.infer_llm_with_tracking(
                    engine=engine,
                    instruction=instruction,
                    exemplars=examples or [],
                    query=final_query,
                    system_info=self.default_system,
                    answer_num=3,
                    max_tokens=8192,
                    temp=temp,
                    use_cache=use_cache,
                    stage="repair",
                    module=self.name,
                )
                # Unwrap if tuple returned (with messages/usage)
                if isinstance(result, tuple):
                    result = result[0]

                # Check if we exceeded timeout
                llm_elapsed = time.time() - llm_start_time
                if llm_elapsed > timeout:
                    self.logger.warning(
                        f"⏱️ LLM call took {llm_elapsed:.2f}s (timeout: {timeout}s) - this may indicate issues"
                    )

                return result
            else:
                result = self.llm.infer_llm(
                    engine=engine,
                    instruction=instruction,
                    exemplars=examples or [],
                    query=final_query,
                    system_info=self.default_system,
                    answer_num=3,
                    max_tokens=8192,
                    temp=temp,
                    use_cache=use_cache,
                )

                # Check if we exceeded timeout
                llm_elapsed = time.time() - llm_start_time
                if llm_elapsed > timeout:
                    self.logger.warning(
                        f"⏱️ LLM call took {llm_elapsed:.2f}s (timeout: {timeout}s) - this may indicate issues"
                    )

                return result
        except Exception as e:
            self.logger.error(f"Error during LLM inference: {e}")
            llm_elapsed = time.time() - llm_start_time
            if llm_elapsed > timeout:
                self.logger.error(
                    f"⏱️ LLM call timed out after {llm_elapsed:.2f}s (timeout: {timeout}s)"
                )
            return []

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the repair module.
        Subclasses must implement the specific repair logic.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to attempt to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        raise NotImplementedError("Repair module subclasses must implement exec() method")
