"""
Module for repairing syntax errors in Verus code.
"""

import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.infer import LLM
from src.modules.baserepair import BaseRepairModule
from src.modules.utils import clean_code, evaluate_samples, get_examples
from src.modules.veval import VerusError, VerusErrorLabel, VerusErrorType, VEval


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
        self.logger.info("Attempting to repair syntax error...")
        code = context.trials[-1].code

        # Syntax errors don't have a specific VerusErrorType, so we can't directly filter by error type
        # Instead, look for compilation errors that might be syntax-related
        if failure_to_fix is None:
            last_trial = context.trials[-1]
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
        if self.is_seq_syntax_error(failure_to_fix, last_trial.eval.rustc_out):
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

        instruction = f"""This code contains a syntax error on line {error_line} in the expression ` {error_text}'. Your mission is to rewrite this expression `{error_text}' to fix the syntax error.

Please make sure to change that wrong expression and do not change any other part of the code. Response with the Rust code only, do not include any explanation. Please use a comment to explain what changes you have made to fix this syntax error."""

        # Add Seq knowledge to help with repair
        seq_examples = self.get_seq_examples()
        seq_knowledge = (
            "Here is the usage for Seq in Verus you can refer:\n```\n{}\n```\n".format(
                "\n".join(seq_examples)
            )
        )
        instruction += "\n\n" + seq_knowledge

        # Load examples
        examples = get_examples(self.config, "seqsyntax", self.logger)

        query_template = "Incorrect line \n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

        query = query_template.format(error_text, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0,
        )

        # Evaluate samples and get the best one
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_seq_syntax",
            logger=self.logger,
        )

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

        instruction = """Your mission is to fix the syntax error in the following Verus code.

Look carefully at the error message and location to identify the syntax issue. Common syntax errors include:
1. Missing or misplaced parentheses, braces, or brackets
2. Missing or incorrect semicolons or commas
3. Incorrect use of operators or methods
4. Incorrect function or method call syntax
5. Incorrect use of generics or type parameters
6. Incorrect use of Verus-specific syntax (like @, spec, proof, etc.)

Fix ONLY the part of the code with the syntax error, and leave the rest unchanged.
Response with the Rust code only, do not include any explanation."""
        instruction += "\n\n" + self.general_knowledge

        # Load examples
        examples = get_examples(self.config, "syntax", self.logger)

        query_template = "Syntax error:\n```\n{}```\n"
        query_template += "\nCode\n```\n{}```\n"

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

        query = query_template.format(error_info, code)

        # Use the llm instance from the base class
        responses = self.llm.infer_llm(
            engine=self.config.get("aoai_debug_model", "gpt-4"),
            instruction=instruction,
            exemplars=examples,
            query=query,
            system_info=self.default_system,
            answer_num=3,
            max_tokens=8192,
            temp=1.0,
        )

        # Evaluate samples and get the best one
        output_dir = Path("output/samples")
        best_code, _, _ = evaluate_samples(
            samples=responses if responses else [code],
            output_dir=output_dir,
            prefix="repair_general_syntax",
            logger=self.logger,
        )

        # Add the best result to context
        context.add_trial(best_code)

        return best_code

    def get_seq_examples(self) -> List[str]:
        """
        Get examples of Seq usage to help with repair.

        Returns:
            List of example Seq usages
        """
        return [
            "// Creating sequences",
            "let s1 = Seq::empty();",
            "let s2 = seq![1, 2, 3];",
            "let s3 = Seq::singleton(42);",
            "",
            "// Getting the length",
            "let len = s2.len();  // 3",
            "",
            "// Indexing (0-based)",
            "let val = s2[1];  // 2",
            "",
            "// Subrange (inclusive start, exclusive end)",
            "let sub = s2.subrange(0, 2);  // seq![1, 2]",
            "",
            "// Concatenation",
            "let s4 = s2 + s3;  // seq![1, 2, 3, 42]",
            "",
            "// Updating a value (returns a new Seq)",
            "let s5 = s2.update(1, 99);  // seq![1, 99, 3]",
            "",
            "// Adding elements (returns a new Seq)",
            "let s6 = s2.push(4);  // seq![1, 2, 3, 4]",
            "",
            "// Converting between Vec and Seq",
            "let v: Vec<int> = vec![1, 2, 3];",
            "let s = v.view();  // Convert Vec to Seq for specifications",
            "",
            "// Filtering elements",
            "let evens = s2.filter(|&x| x % 2 == 0);  // seq![2]",
            "",
            "// Taking the first n elements",
            "let first_two = s2.take(2);  // seq![1, 2]",
        ]
