"""
Module for repairing missing elements in Verus code.
"""

import logging
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.utils.path_utils import best_dir, samples_dir


class RepairMissingModule(BaseRepairModule):
    """
    Module for repairing missing elements.
    Handles missing imports, trait implementations, and other missing elements.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_missing",
            desc="Repair missing imports, implementations, and other elements",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the missing elements repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("Attempting to repair missing element error...")
        code = context.trials[-1].code

        # If a specific failure isn't provided, try to get one from the last trial
        if failure_to_fix is None:
            last_trial = context.trials[-1]
            import_failures = last_trial.eval.get_failures(error_type=VerusErrorType.MissingImport)
            impl_failures = last_trial.eval.get_failures(error_type=VerusErrorType.MissImpl)
            failures = import_failures + impl_failures

            if not failures:
                self.logger.warning("No missing element failures found in the last trial.")
                return code  # Return original code if no missing element error

            failure_to_fix = self.get_one_failure(failures)
            if not failure_to_fix:
                self.logger.warning("Could not select a failure to fix.")
                return code

        # Choose appropriate method based on error type
        if failure_to_fix.error == VerusErrorType.MissingImport:
            return self.repair_missing_import(context, failure_to_fix)
        elif failure_to_fix.error == VerusErrorType.MissImpl:
            return self.repair_missing_impl(context, failure_to_fix)
        else:
            self.logger.warning(
                f"Unsupported error type: {failure_to_fix.error.name}. Skipping repair."
            )
            return code

    def repair_missing_import(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair missing import errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        instruction = """Your mission is to fix the missing import error for the following code.
For example, Verus might report:
"cannot find macro verus in this scope"
To fix this, you need to add a suitable `use` statement at the TOP of the file, OUTSIDE and BEFORE the `verus!` macro:

CORRECT format:
```
use vstd::prelude::*;

verus! {
    // code here
}
```

INCORRECT format (DO NOT do this):
```
verus! {
    use vstd::prelude::*;  // WRONG - imports must be outside verus! macro
    // code here
}
```

Important rules:
1. Place ALL import statements (`use` statements) at the very top of the file
2. Imports must be OUTSIDE and BEFORE the `verus!` macro block
3. Add a `main` function inside the `verus!` block if it does not already have one
4. Respond with the entire Rust code only (no explanations) after fixing the import issue."""
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        # Load examples
        examples = get_examples(self.config, "import", self.logger)

        query_template = "Missing import error:\n```\n{}```\n"
        query_template += "\nCode:\n```\n{}```\n"

        if failure_to_fix.trace:
            # Take the first trace to show the relevant snippet
            err_text = failure_to_fix.trace[0].get_text(snippet=False)
        else:
            err_text = failure_to_fix.error_text

        query = query_template.format(err_text, code)

        # Use tracking wrapper for LLM calls

        if context is not None and hasattr(context, "infer_llm_with_tracking"):
            result = context.infer_llm_with_tracking(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
                stage="repair",
                module=self.name,
            )

            responses = result[0] if isinstance(result, tuple) else result

        else:
            responses = self.llm.infer_llm(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
            )
        # Evaluate samples and get the best one
        output_dir = samples_dir()
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_missing_import",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_missing_impl(self, context, failure_to_fix: VerusError) -> str:
        """
        Repair missing implementation errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code

        # Extract missing function names from the error
        missing_funcs = failure_to_fix.get_miss_impl_funcs()
        missing_funcs_str = ", ".join([f"`{func}`" for func in missing_funcs])

        instruction = f"""Your mission is to fix the error where not all trait items are implemented in the following Verus code.

The missing method(s): {missing_funcs_str}

For each missing method, you need to implement it based on the trait definition and the structure of the existing code. Make sure your implementation:
1. Meets the required method signature
2. Follows the same style as other method implementations
3. Maintains any necessary invariants or properties of the data structure
4. Handles any edge cases appropriately
5. Includes appropriate ensures/requires clauses if needed

Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge + "\n\n" + context.gen_knowledge()

        # Load examples
        examples = get_examples(self.config, "impl", self.logger)

        query_template = "Missing implementation error:\n```\n{}```\n"
        query_template += "\nCode:\n```\n{}```\n"

        if failure_to_fix.trace:
            # Take the first trace to show the relevant snippet
            err_text = failure_to_fix.trace[0].get_text(snippet=False)
        else:
            err_text = failure_to_fix.error_text

        query = query_template.format(err_text, code)

        # Use tracking wrapper for LLM calls

        if context is not None and hasattr(context, "infer_llm_with_tracking"):
            result = context.infer_llm_with_tracking(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
                stage="repair",
                module=self.name,
            )

            responses = result[0] if isinstance(result, tuple) else result

        else:
            responses = self.llm.infer_llm(
                engine=self.config.get("aoai_generation_model", "gpt-4"),
                instruction=instruction,
                exemplars=examples,
                query=query,
                system_info=self.default_system,
                answer_num=3,
                max_tokens=8192,
                temp=1.0,
            )
        # Evaluate samples and get the best one
        output_dir = samples_dir()
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_missing_impl",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code
