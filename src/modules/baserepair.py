"""
Base class for Repair modules in VerusAgent.
"""

import logging
from typing import Any, Dict, List, Optional

from infer import LLM
from modules.base import BaseModule
from modules.veval import VerusError, VerusErrorType, VEval


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
        super().__init__(name=name, desc=desc)
        self.config = config
        self.logger = logger
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs if immutable_funcs is not None else []

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
        raise NotImplementedError(
            "Repair module subclasses must implement exec() method"
        )
