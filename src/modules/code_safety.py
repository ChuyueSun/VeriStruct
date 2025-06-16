# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #

import tempfile
import os
from pathlib import Path
from typing import List, Optional, Tuple

from src.modules.base import BaseModule
from src.modules.lynette import lynette
from src.modules.utils import remove_rust_comments, get_func_body
from src.context import Context


class CodeSafetyModule(BaseModule):
    """
    Module that uses Lynette to validate code changes are semantically safe.
    This module can be inserted between other modules to ensure transformations
    preserve program semantics.
    """

    def __init__(self, config, logger):
        super().__init__(config, logger)
        self.name = "code_safety"
        self.desc = "Validates that code changes preserve semantic equivalence using Lynette"
        self.immutable_funcs = config.get("immutable_functions", [])

    def exec(self, context: Context) -> str:
        """
        Validate that the latest code change is semantically safe.
        If unsafe, attempt to revert to the previous safe version.
        """
        if len(context.trials) < 2:
            self.logger.info("Not enough trials to perform safety validation")
            return context.get_latest_code()

        current_code = context.get_latest_code()
        previous_code = context.trials[-2].code

        self.logger.info("Validating code safety using Lynette...")
        
        is_safe = self._validate_code_safety(
            previous_code, 
            current_code, 
            self.immutable_funcs
        )

        if is_safe:
            self.logger.info("Code change validated as safe")
            return current_code
        else:
            self.logger.warning("Code change detected as unsafe - reverting to previous version")
            # Add the reverted code as a new trial
            context.add_trial(previous_code)
            return previous_code

    def _validate_code_safety(
        self, 
        origin_code: str, 
        changed_code: str, 
        immutable_funcs: List[str]
    ) -> bool:
        """
        Use Lynette to validate that code changes are semantically safe.
        
        Args:
            origin_code: Original code
            changed_code: Modified code  
            immutable_funcs: List of function names that should not be modified
            
        Returns:
            True if changes are safe, False otherwise
        """
        # Check immutable functions haven't changed
        for func_name in immutable_funcs:
            origin_body = get_func_body(origin_code, func_name, logger=self.logger)
            changed_body = get_func_body(changed_code, func_name, logger=self.logger)

            if origin_body is None or changed_body is None:
                self.logger.warning(f"Could not extract function '{func_name}' for safety check")
                return False

            origin_clean = remove_rust_comments(origin_body)
            changed_clean = remove_rust_comments(changed_body)

            if origin_clean != changed_clean:
                self.logger.error(f"Immutable function '{func_name}' was modified")
                return False

        # Use Lynette's compare functionality
        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, prefix="safety_orig_", suffix=".rs"
            ) as orig_f, tempfile.NamedTemporaryFile(
                mode="w", delete=False, prefix="safety_changed_", suffix=".rs"
            ) as changed_f:
                
                orig_f.write(origin_code)
                orig_f.flush()
                changed_f.write(changed_code)
                changed_f.flush()

                # Use Lynette's compare command
                result = lynette.run(["compare", "-t", orig_f.name, changed_f.name])

                if result.returncode == 0:
                    return True
                elif result.returncode == 1:
                    err_msg = result.stdout.strip()
                    if err_msg == "Files are different":
                        self.logger.info("Lynette detected semantic differences")
                        return False
                    else:
                        self.logger.error(f"Lynette comparison error: {err_msg}")
                        return False
                else:
                    self.logger.error(f"Lynette comparison failed: {result.stderr}")
                    return False

        except Exception as e:
            self.logger.error(f"Exception during safety validation: {e}")
            return False
        finally:
            # Clean up temp files
            if 'orig_f' in locals() and os.path.exists(orig_f.name):
                os.unlink(orig_f.name)
            if 'changed_f' in locals() and os.path.exists(changed_f.name):
                os.unlink(changed_f.name)

    def validate_between_trials(
        self, 
        context: Context, 
        trial_idx1: int, 
        trial_idx2: int
    ) -> bool:
        """
        Validate safety between two specific trials.
        
        Args:
            context: Current context
            trial_idx1: Index of first trial
            trial_idx2: Index of second trial
            
        Returns:
            True if transition is safe
        """
        if trial_idx1 >= len(context.trials) or trial_idx2 >= len(context.trials):
            return False
            
        code1 = context.trials[trial_idx1].code
        code2 = context.trials[trial_idx2].code
        
        return self._validate_code_safety(code1, code2, self.immutable_funcs) 