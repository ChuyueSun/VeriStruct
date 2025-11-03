"""
Module for repairing syntax errors in Verus code.
"""

import logging
import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval
from src.prompts.template import build_instruction
from src.utils.path_utils import best_dir, debug_dir, samples_dir


def _remove_ret_from_proof_blocks(code: str) -> str:
    """
    Remove or comment out lines that reference 'ret' inside proof blocks.
    The 'ret' variable is only available in ensures clauses, not in function body proof blocks.

    Args:
        code: The source code with potential ret references in proof blocks

    Returns:
        Code with ret references removed from proof blocks
    """
    lines = code.split("\n")
    result_lines = []
    in_proof_block = False
    proof_block_depth = 0

    for i, line in enumerate(lines):
        stripped = line.strip()

        # Track proof block entry/exit
        if stripped.startswith("proof {"):
            in_proof_block = True
            proof_block_depth = 1
            result_lines.append(line)
            continue

        if in_proof_block:
            # Count braces to track nested blocks
            proof_block_depth += line.count("{") - line.count("}")

            # Exit proof block when depth reaches 0
            if proof_block_depth <= 0:
                in_proof_block = False
                result_lines.append(line)
                continue

            # Check if this line references 'ret' (as a variable, not in a comment)
            # Skip if it's in a comment
            code_part = line.split("//")[0]  # Remove comment portion
            if re.search(r"\bret\b", code_part):
                # Comment out this line instead of removing it
                result_lines.append(
                    f"{line[:len(line) - len(line.lstrip())]}// REMOVED: {stripped} // Error: ret not in scope in proof blocks"
                )
                continue

        result_lines.append(line)

    return "\n".join(result_lines)


# Pattern-based syntax fixes for common Verus errors
VERUS_SYNTAX_PATTERNS = {
    "assert_forall_missing_by": {
        "error_keywords": ["expected `by`"],
        "pattern": r"(assert forall\|[^|]+\|[^;]+);",
        "fix": lambda code: re.sub(
            r"(assert forall\|[^|]+\|[^;]+);", r"\1 by {\n    \n}", code
        ),
        "description": "Add missing 'by {}' clause to assert forall",
    },
    "assert_forall_implies": {
        "error_keywords": ["expected `by`", "unexpected token"],
        "pattern": r"(assert forall\|[^|]+\|[^=]+)==>",
        "fix": lambda code: re.sub(
            r"(assert forall\|[^|]+\|[^=]+)==>", r"\1implies", code
        ),
        "description": "Replace '==>' with 'implies' in assert forall",
    },
    "map_equality": {
        "error_keywords": ["postcondition not satisfied", "precondition not satisfied"],
        "pattern": r"\.to_map\(\)\s*==\s*",
        "fix": lambda code: re.sub(r"([\w.]+\.to_map\(\))\s*==\s*", r"\1 =~= ", code),
        "description": "Use =~= for map equality instead of ==",
    },
    "ret_in_proof_block": {
        "error_keywords": ["cannot find value `ret`", "not found in this scope"],
        "pattern": r"\bret\b",
        "fix": lambda code: _remove_ret_from_proof_blocks(code),
        "description": "Remove references to 'ret' variable inside proof blocks (ret is only available in ensures)",
    },
}


def apply_pattern_fixes(code: str, error_text: str, logger) -> str:
    """
    Apply pattern-based fixes for common Verus syntax errors.

    Args:
        code: The code with syntax errors
        error_text: The error message from compiler
        logger: Logger for reporting fixes

    Returns:
        Fixed code or original if no patterns match
    """
    for pattern_name, pattern_info in VERUS_SYNTAX_PATTERNS.items():
        # Check if error message matches this pattern's keywords
        if any(keyword in error_text for keyword in pattern_info["error_keywords"]):
            # Check if the pattern exists in the code
            if re.search(pattern_info["pattern"], code):
                logger.info(f"Applying pattern fix: {pattern_info['description']}")
                fixed_code = pattern_info["fix"](code)
                if fixed_code != code:
                    logger.info(f"Successfully applied {pattern_name} pattern fix")
                    return fixed_code

    return code


class RepairSyntaxModule(BaseRepairModule):
    """
    Module for repairing syntax errors.
    Specialized in handling syntax errors, particularly those related to Seq operations.
    """

    def __init__(self, config, logger, immutable_funcs=[]):
        super().__init__(
            name="repair_syntax",
            desc="Repair syntax errors, including Seq-related syntax issues",
            config=config,
            logger=logger,
            immutable_funcs=immutable_funcs,
        )

    def exec(self, context, failure_to_fix: Optional[VerusError] = None) -> str:
        """
        Execute the syntax repair module.

        Args:
            context: The current execution context
            failure_to_fix: The specific VerusError to fix (optional)

        Returns:
            The potentially repaired code string.
        """
        self.logger.info("=" * 60)
        self.logger.info("SYNTAX REPAIR - Starting")
        self.logger.info("=" * 60)

        # Always reference the latest trial; needed later even if failure_to_fix is provided
        last_trial = context.trials[-1]
        code = last_trial.code

        self.logger.info(f"Current score: {last_trial.eval.get_score()}")

        # Syntax errors don't have a specific VerusErrorType, so we can't directly filter by error type
        # Instead, look for compilation errors that might be syntax-related
        if failure_to_fix is None:
            if last_trial.eval.compilation_error:
                # For syntax errors, we need to look at the raw output rather than parsed errors
                if "error[E0433]: failed to resolve" in last_trial.eval.rustc_out:
                    self.logger.info(
                        "Detected potential name resolution error, will try syntax repair"
                    )
                    # Try to find a relevant error
                    failures = last_trial.eval.verus_errors
                    if failures:
                        failure_to_fix = self.get_one_failure(failures)
                elif (
                    "unexpected token" in last_trial.eval.rustc_out
                    or "expected" in last_trial.eval.rustc_out
                ):
                    self.logger.info(
                        "Detected potential syntax error, will try syntax repair"
                    )
                    # Try to find a relevant error
                    failures = last_trial.eval.verus_errors
                    if failures:
                        failure_to_fix = self.get_one_failure(failures)
                else:
                    self.logger.warning(
                        "No specific syntax errors found in the compilation output."
                    )
                    return code
            else:
                self.logger.warning(
                    "No compilation errors detected, skipping syntax repair."
                )
                return code

        # Check if we're dealing with Seq-related syntax
        is_seq_error = self.is_seq_syntax_error(
            failure_to_fix, last_trial.eval.rustc_out
        )
        self.logger.info(
            f"Error classification: {'Seq-related' if is_seq_error else 'General'} syntax error"
        )

        if is_seq_error:
            return self.repair_seq_syntax_error(context, failure_to_fix)
        else:
            return self.repair_general_syntax_error(
                context, failure_to_fix, last_trial.eval.rustc_out
            )

    def is_seq_syntax_error(
        self, failure: Optional[VerusError], rustc_out: str
    ) -> bool:
        """
        Determine if the error is related to Seq syntax.

        Args:
            failure: The VerusError to check
            rustc_out: The raw rustc output for additional context

        Returns:
            True if this is a Seq-related syntax error, False otherwise
        """
        if failure is None:
            return False

        seq_related_terms = [
            "Seq",
            "seq!",
            "verus::seq",
            "seq::",
            "vec.view()",
            ".subrange(",
            ".filter(",
            ".take(",
            ".push(",
            ".update(",
        ]

        # Check the error message and trace for Seq-related terms
        error_text = failure.error_text
        for term in seq_related_terms:
            if term in error_text:
                return True

        # If there are traces, check them too
        if failure.trace:
            for trace in failure.trace:
                if hasattr(trace, "get_text"):
                    trace_text = trace.get_text()
                    for term in seq_related_terms:
                        if term in trace_text:
                            return True

        # Check the raw rustc output as a fallback
        for term in seq_related_terms:
            if term in rustc_out:
                return True

        return False

    def repair_seq_syntax_error(
        self, context, failure_to_fix: Optional[VerusError]
    ) -> str:
        """
        Repair Seq-related syntax errors.
        This is based on the repair_SeqSyntax_error function from refinement.py.

        Args:
            context: The current execution context
            failure_to_fix: The specific error to fix (if available)

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Extract error information
        error_line = None
        error_text = None

        if failure_to_fix and failure_to_fix.trace:
            error_trace = failure_to_fix.trace[0]
            error_line = error_trace.lines[0]
            error_text = error_trace.get_text().strip()

        if not error_line or not error_text:
            # If we couldn't extract error information, use a more generic approach
            return self.repair_general_syntax_error(
                context, failure_to_fix, context.trials[-1].eval.rustc_out
            )

        # Base instruction for Seq syntax repair
        base_instruction = f"""This code contains a syntax error on line {error_line} in the expression ` {error_text}'. Your mission is to rewrite this expression `{error_text}' to fix the syntax error.

Please make sure to change that wrong expression and do not change any other part of the code. Response with the Rust code only, do not include any explanation. Please use a comment to explain what changes you have made to fix this syntax error."""

        # Add Seq knowledge to help with repair
        seq_examples = self.get_seq_examples()
        seq_knowledge = (
            "Here is the usage for Seq in Verus you can refer:\n```\n{}\n```\n".format(
                "\n".join(seq_examples)
            )
        )
        base_instruction += "\n\n" + seq_knowledge

        query_template = "Incorrect line \n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"
        query = query_template.format(error_text, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info("-" * 50)
            self.logger.info(
                f"Seq syntax repair attempt {retry_attempt + 1}/{max_retries}"
            )
            self.logger.info("-" * 50)

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "seqsyntax", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_seq_syntax_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved seq syntax repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_seq_syntax_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the Seq syntax error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def repair_general_syntax_error(
        self, context, failure_to_fix: Optional[VerusError], rustc_out: str
    ) -> str:
        """
        Repair general syntax errors.

        Args:
            context: The current execution context
            failure_to_fix: The specific error to fix (if available)
            rustc_out: The raw rustc output

        Returns:
            The potentially repaired code string.
        """
        code = context.trials[-1].code
        original_code = code  # Store original for safety checking

        # Base instruction for syntax repair
        base_instruction = """Your mission is to fix the syntax error in the following Verus code.

Look carefully at the error message and location to identify the syntax issue. Common syntax errors include:
1. Missing or misplaced parentheses, braces, or brackets
2. Missing or incorrect semicolons or commas
3. Incorrect use of operators or methods
4. Incorrect function or method call syntax
5. Incorrect use of generics or type parameters
6. Incorrect use of Verus-specific syntax (like @, spec, proof, etc.)

Fix ONLY the part of the code with the syntax error, and leave the rest unchanged.
Response with the Rust code only, do not include any explanation."""

        # Extract relevant error information
        error_info = ""
        if failure_to_fix:
            error_info += failure_to_fix.error_text + "\n"
            if failure_to_fix.trace:
                for trace in failure_to_fix.trace:
                    if hasattr(trace, "get_text"):
                        error_info += trace.get_text() + "\n"

        # Include relevant parts of rustc_out to help identify the error
        error_lines = []
        for line in rustc_out.splitlines():
            if "error" in line:
                error_lines.append(line)
            elif "--> " in line or line.strip().startswith("|"):
                error_lines.append(line)

        if error_lines:
            error_info += "\n" + "\n".join(error_lines[:20])  # Limit to first 20 lines

        # Normalize variable tmp paths to a stable placeholder so prompts are identical across runs
        normalized_error_info = re.sub(
            r"/tmp/tmp[0-9A-Za-z_\-]+", "<TMP_PATH>", error_info
        )

        query_template = "Syntax error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"
        query = query_template.format(normalized_error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info("-" * 50)
            self.logger.info(f"Syntax repair attempt {retry_attempt + 1}/{max_retries}")
            self.logger.info("-" * 50)

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "syntax", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_general_syntax_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved syntax repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_general_syntax_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the syntax error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def get_seq_examples(self) -> List[str]:
        """
        Get examples of Seq usage from examples/seq/*.rs to help with repair.

        Returns:
            List of example Seq usages
        """
        examples_dir = os.path.join(
            os.path.dirname(os.path.dirname(__file__)), "examples", "seq"
        )
        examples = []
        try:
            for file in os.listdir(examples_dir):
                if file.endswith(".rs"):
                    file_path = os.path.join(examples_dir, file)
                    with open(file_path, "r") as f:
                        examples.extend(line.strip() for line in f if line.strip())
            return examples
        except Exception as e:
            print(f"Warning: Could not load sequence examples from {examples_dir}: {e}")
            # Return an empty list as fallback
            return []

            error_info += failure_to_fix.error_text + "\n"
            if failure_to_fix.trace:
                for trace in failure_to_fix.trace:
                    if hasattr(trace, "get_text"):
                        error_info += trace.get_text() + "\n"

        # Include relevant parts of rustc_out to help identify the error
        error_lines = []
        for line in rustc_out.splitlines():
            if "error" in line:
                error_lines.append(line)
            elif "--> " in line or line.strip().startswith("|"):
                error_lines.append(line)

        if error_lines:
            error_info += "\n" + "\n".join(error_lines[:20])  # Limit to first 20 lines

        # Normalize variable tmp paths to a stable placeholder so prompts are identical across runs
        normalized_error_info = re.sub(
            r"/tmp/tmp[0-9A-Za-z_\-]+", "<TMP_PATH>", error_info
        )

        query_template = "Syntax error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"
        query = query_template.format(normalized_error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info("-" * 50)
            self.logger.info(f"Syntax repair attempt {retry_attempt + 1}/{max_retries}")
            self.logger.info("-" * 50)

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "syntax", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_general_syntax_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved syntax repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_general_syntax_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the syntax error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def get_seq_examples(self) -> List[str]:
        """
        Get examples of Seq usage from examples/seq/*.rs to help with repair.

        Returns:
            List of example Seq usages
        """
        examples_dir = os.path.join(
            os.path.dirname(os.path.dirname(__file__)), "examples", "seq"
        )
        examples = []
        try:
            for file in os.listdir(examples_dir):
                if file.endswith(".rs"):
                    file_path = os.path.join(examples_dir, file)
                    with open(file_path, "r") as f:
                        examples.extend(line.strip() for line in f if line.strip())
            return examples
        except Exception as e:
            print(f"Warning: Could not load sequence examples from {examples_dir}: {e}")
            # Return an empty list as fallback
            return []

            error_info += failure_to_fix.error_text + "\n"
            if failure_to_fix.trace:
                for trace in failure_to_fix.trace:
                    if hasattr(trace, "get_text"):
                        error_info += trace.get_text() + "\n"

        # Include relevant parts of rustc_out to help identify the error
        error_lines = []
        for line in rustc_out.splitlines():
            if "error" in line:
                error_lines.append(line)
            elif "--> " in line or line.strip().startswith("|"):
                error_lines.append(line)

        if error_lines:
            error_info += "\n" + "\n".join(error_lines[:20])  # Limit to first 20 lines

        # Normalize variable tmp paths to a stable placeholder so prompts are identical across runs
        normalized_error_info = re.sub(
            r"/tmp/tmp[0-9A-Za-z_\-]+", "<TMP_PATH>", error_info
        )

        query_template = "Syntax error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"
        query = query_template.format(normalized_error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info("-" * 50)
            self.logger.info(f"Syntax repair attempt {retry_attempt + 1}/{max_retries}")
            self.logger.info("-" * 50)

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "syntax", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_general_syntax_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved syntax repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_general_syntax_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the syntax error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def get_seq_examples(self) -> List[str]:
        """
        Get examples of Seq usage from examples/seq/*.rs to help with repair.

        Returns:
            List of example Seq usages
        """
        examples_dir = os.path.join(
            os.path.dirname(os.path.dirname(__file__)), "examples", "seq"
        )
        examples = []
        try:
            for file in os.listdir(examples_dir):
                if file.endswith(".rs"):
                    file_path = os.path.join(examples_dir, file)
                    with open(file_path, "r") as f:
                        examples.extend(line.strip() for line in f if line.strip())
            return examples
        except Exception as e:
            print(f"Warning: Could not load sequence examples from {examples_dir}: {e}")
            # Return an empty list as fallback
            return []

            error_info += failure_to_fix.error_text + "\n"
            if failure_to_fix.trace:
                for trace in failure_to_fix.trace:
                    if hasattr(trace, "get_text"):
                        error_info += trace.get_text() + "\n"

        # Include relevant parts of rustc_out to help identify the error
        error_lines = []
        for line in rustc_out.splitlines():
            if "error" in line:
                error_lines.append(line)
            elif "--> " in line or line.strip().startswith("|"):
                error_lines.append(line)

        if error_lines:
            error_info += "\n" + "\n".join(error_lines[:20])  # Limit to first 20 lines

        # Normalize variable tmp paths to a stable placeholder so prompts are identical across runs
        normalized_error_info = re.sub(
            r"/tmp/tmp[0-9A-Za-z_\-]+", "<TMP_PATH>", error_info
        )

        query_template = "Syntax error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"
        query = query_template.format(normalized_error_info, code)

        max_retries = 3
        safe_responses = []

        for retry_attempt in range(max_retries):
            self.logger.info("-" * 50)
            self.logger.info(f"Syntax repair attempt {retry_attempt + 1}/{max_retries}")
            self.logger.info("-" * 50)

            # Build complete instruction using the prompt system
            instruction = build_instruction(
                base_instruction=base_instruction,
                add_common=True,  # Add common Verus knowledge
                code=code,  # For Seq detection
                knowledge=self.general_knowledge,  # Add general knowledge
            )

            # Load examples
            examples = get_examples(self.config, "syntax", self.logger)

            # Ensure debug directory exists for prompt saving
            dbg_dir = debug_dir()
            prompt_path2 = (
                dbg_dir / f"repair_general_syntax_prompt_{len(context.trials)}.txt"
            )
            prompt_path2.write_text(instruction + "\n\n---\n\n" + query)
            self.logger.info(f"Saved syntax repair prompt to {prompt_path2}")

            # Get responses from LLM
            responses = self._get_llm_responses(
                instruction,
                query,
                examples,
                retry_attempt=retry_attempt,
                use_cache=True,
                #   use_cache=(retry_attempt == 0),
                context=context,  # Pass context for appending knowledge
            )

            if not responses and retry_attempt == max_retries - 1:
                return code

            # Evaluate samples and get the best one with safety checking
            output_dir = samples_dir()
            best_code = self.evaluate_repair_candidates(
                original_code=code,
                candidates=responses if responses else [code],
                output_dir=output_dir,
                prefix=f"repair_general_syntax_attempt_{retry_attempt + 1}",
            )

            if best_code != code:  # If we got a potentially better solution
                safe_responses.append(best_code)
                self.logger.info(
                    f"Found a potentially safe response after {retry_attempt + 1} attempts"
                )
                break

            if retry_attempt < max_retries - 1:
                base_instruction += (
                    f"\n\nIMPORTANT: Previous attempt failed to fix the syntax error. "
                    f"Please try a different approach. Attempt {retry_attempt + 2}/{max_retries}."
                )

        # If no safe responses found after all retries, fall back to original
        if not safe_responses:
            self.logger.warning(
                "No safe responses found after all retries, using original code"
            )
            return code

        # Use the last safe response (since we break after finding one)
        best_code = safe_responses[-1]

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def get_seq_examples(self) -> List[str]:
        """
        Get examples of Seq usage from examples/seq/*.rs to help with repair.

        Returns:
            List of example Seq usages
        """
        examples_dir = os.path.join(
            os.path.dirname(os.path.dirname(__file__)), "examples", "seq"
        )
        examples = []
        try:
            for file in os.listdir(examples_dir):
                if file.endswith(".rs"):
                    file_path = os.path.join(examples_dir, file)
                    with open(file_path, "r") as f:
                        examples.extend(line.strip() for line in f if line.strip())
            return examples
        except Exception as e:
            print(f"Warning: Could not load sequence examples from {examples_dir}: {e}")
            # Return an empty list as fallback
            return []
