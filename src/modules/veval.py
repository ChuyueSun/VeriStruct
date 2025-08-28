# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #


import json
import logging
import os
import re
import subprocess
import tempfile
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import List, Optional, Set

from src.configs.sconfig import config, reset_config


class VerusErrorType(Enum):
    PreCondFail = 1
    PostCondFail = 2
    InvFailEnd = 3
    InvFailFront = 4
    DecFailEnd = 5
    DecFailCont = 6
    TestAssertFail = 7
    RecommendNotMet = 8
    AssertFail = 11
    ArithmeticFlow = 12
    MismatchedType = 13
    PreCondFailVecLen = 14
    MissImpl = 15
    Other = 16
    ensure_private = 17
    require_private = 18
    MissingImport = 19
    TypeAnnotation = 20
    ConstructorFailTypeInvariant = 21
    CannotCallFunc = 22
    RequiresOldSelf = 23
    PubSpecVisibility = 24


m2VerusError = {
    "precondition not satisfied": VerusErrorType.PreCondFail,
    "postcondition not satisfied": VerusErrorType.PostCondFail,
    "invariant not satisfied at end of loop body": VerusErrorType.InvFailEnd,
    "invariant not satisfied before loop": VerusErrorType.InvFailFront,
    "decreases not satisfied at end of loop": VerusErrorType.DecFailEnd,
    "decreases not satisfied at continue": VerusErrorType.DecFailCont,
    "recommendation not met": VerusErrorType.RecommendNotMet,
    "assertion failed": VerusErrorType.AssertFail,
    "possible arithmetic underflow/overflow": VerusErrorType.ArithmeticFlow,
    "mismatched types": VerusErrorType.MismatchedType,
    "in 'ensures' clause of public function, cannot access any field of a datatype where one or more fields are private": VerusErrorType.ensure_private,
    "in 'requires' clause of public function, cannot refer to private function": VerusErrorType.require_private,
    "cannot find macro `verus` in this scope": VerusErrorType.MissingImport,
    "type annotations needed": VerusErrorType.TypeAnnotation,
    "constructed value may fail to meet its declared type invariant": VerusErrorType.ConstructorFailTypeInvariant,
    "cannot call function": VerusErrorType.CannotCallFunc,
    "in requires, use `old(self)` to refer to the pre-state of an &mut variable": VerusErrorType.RequiresOldSelf,
    "non-private spec function must be marked open or closed": VerusErrorType.PubSpecVisibility,
}

VerusError2m = {v: k for k, v in m2VerusError.items()}


class VerusErrorLabel(Enum):
    NullLabel = 0
    FailedThisPostCond = 1
    FailedThisPreCond = 2
    RecmdNotMet = 3
    EndOfFunc = 4


m2VerusErrorLabel = {
    None: VerusErrorLabel.NullLabel,
    "failed this postcondition": VerusErrorLabel.FailedThisPostCond,
    "failed precondition": VerusErrorLabel.FailedThisPreCond,
    "recommendation not met": VerusErrorLabel.RecmdNotMet,
    "at the end of the function body": VerusErrorLabel.EndOfFunc,
}

VerusErrorLabel2m = {v: k for k, v in m2VerusErrorLabel.items()}


class Verus:
    def __init__(self):
        self.verus_path = None

    def set_verus_path(self, path):
        self.verus_path = os.path.realpath(path)
        self.vstd_path = os.path.realpath(
            os.path.join(self.verus_path, "../../../vstd/")
        )
        # print(f"verus path: {self.verus_path}")
        # print(f"vstd path: {self.vstd_path}")


verus = Verus()


class ErrorText:
    def __init__(self, text):
        self.text = text["text"]
        self.hl_start = text["highlight_start"]
        self.hl_end = text["highlight_end"]


class ErrorTrace:
    def __init__(self, span):
        self.fname = span["file_name"]
        self.lines = (int(span["line_start"]), int(span["line_end"]))
        if span["label"] not in m2VerusErrorLabel:
            self.label = VerusErrorLabel.NullLabel
        else:
            self.label = m2VerusErrorLabel[span["label"]]
        self.text = [ErrorText(t) for t in span["text"]]
        self.vstd_err = self.fname.startswith(os.path.realpath(verus.vstd_path))
        self.strlabel = span["label"]

    def is_vstd_err(self):
        return self.vstd_err

    def get_text(self, snippet=True, pre=4, post=2):
        ret = (
            f"{VerusErrorLabel2m[self.label]}\n"
            if VerusErrorLabel2m[self.label]
            else ""
        )
        if not snippet or len(self.text) <= pre + post + 1:
            return ret + "\n".join([t.text for t in self.text])
        else:
            return ret + "\n".join(
                [t.text for t in self.text[:pre]]
                + ["..."]
                + [t.text for t in self.text[-post:]]
            )

    # TO be refined
    def get_highlights(self):
        return [t.text[t.hl_start - 1 : t.hl_end - 1] for t in self.text]

    def get_lines(self):
        return self.lines


class VerusError:
    def __init__(self, err: dict):
        # Store the raw message text and spans
        self.error_text = err["message"]
        self.spans = err["spans"] if "spans" in err else []
        self.logger = logging.getLogger("VerusError")

        # Create the trace first so we can use it for error classification
        self.trace = [ErrorTrace(t) for t in self.spans]  # Bottom-up stack trace

        # Get the full error message including span labels
        if self.spans:
            span_labels = [
                span.get("label", "") for span in self.spans if "label" in span
            ]
            self.error_text = f"{self.error_text} ({'; '.join(label for label in span_labels if label)})"

        # Default to 'Other' unless a partial match is found
        self.error = VerusErrorType.Other

        # Try to match by substring against known keys
        for known_msg, err_type in m2VerusError.items():
            if known_msg in self.error_text:
                # Special case: don't treat empty function body errors as type errors
                if err_type == VerusErrorType.MismatchedType:
                    if "implicitly returns `()`" in self.error_text:
                        continue
                self.error = err_type
                break

        # Handle any special-cases not captured in the dictionary
        if self.error == VerusErrorType.Other:
            if "not all trait items implemented, missing" in self.error_text:
                self.error = VerusErrorType.MissImpl
        
        # Special case: TestAssertFail is an assertion failure inside a test function
        if self.error == VerusErrorType.AssertFail:
            self.logger.info(f"Found assertion failure, checking if it's in a test function...")
            for trace in self.trace:
                trace_text = trace.get_text().lower()
                if trace.fname:
                    self.logger.info(f"Checking trace from file {trace.fname}")
                    self.logger.info(f"Trace text: {trace_text}")
                    if "test" in trace_text:
                        self.logger.info("Found 'test' in trace, classifying as TestAssertFail")
                        self.error = VerusErrorType.TestAssertFail
                        break



        # a subtype of precondfail that often requires separate treatment
        if self.error == VerusErrorType.PreCondFail:
            if self.trace and "i < vec.view().len()" in self.trace[0].get_text():
                self.error = VerusErrorType.PreCondFailVecLen

    def __str__(self):
        return f"{self.error}: {self.error_text}"

    def get_miss_impl_funcs(self):
        if self.error != VerusErrorType.MissImpl:
            return []

        def extract_function_names(text):
            pattern = r"`(\w+)`"
            matches = re.findall(pattern, text)
            return matches

        function_names = extract_function_names(self.error_text)
        return function_names

    def get_text(self, snippet=True, pre=4, post=2, topdown=True):
        traces = []
        for t in self.trace:
            t_text = t.get_text(snippet, pre, post)
            if t_text and t_text not in traces:
                traces.append(t_text)

        if topdown:
            traces = traces[::-1]

        span_texts = []
        for span in self.spans:
            if "text" in span:
                highlights = []
                for t in span["text"]:
                    text = t["text"][t["highlight_start"] - 1 : t["highlight_end"] - 1]
                    highlights.append(text)
                highlight_text = " ".join(highlights)
                label = span["label"]
                span_texts += [f"{label}: {highlight_text}"]
        return "\n".join(traces) + "\n  " + "\n  ".join(span_texts)

    def __eq__(self, value: object) -> bool:
        if not isinstance(value, VerusError):
            return False

        return (
            self.error_text == value.error_text and self.get_text() == value.get_text()
        )


class EvalScore:
    def __init__(
        self, verified: int, errors: int, compilation_error: bool, verus_errors: int = 0
    ):
        self.compilation_error = compilation_error
        self.verified = verified
        self.errors = errors
        if self.verified == self.errors == 0:
            self.compilation_error = True
            self.verified = -1
            self.errors = 999
        self.verus_errors = verus_errors

    @staticmethod
    def get_worst_score() -> object:
        return EvalScore(-10000, 10000, True, 10000)

    def is_correct(self) -> bool:
        if self.verified < 0:
            return False
        return (
            self.verified > 0
            and self.errors == 0
            and not self.compilation_error
            and self.verus_errors == 0
        )

    def is_good_repair(self, value: object) -> bool:
        # Check whether self is a good repair to value
        if not isinstance(value, EvalScore):
            return False

        # Compilation error is the highest priority - a repair that causes compilation errors
        # is NEVER an improvement over code that compiles
        if self.compilation_error != value.compilation_error:
            return not self.compilation_error

        # For code that both compile or both fail to compile
        # Consider it an improvement if more functions are verified
        if self.verified > value.verified:
            return True

        # If same number of verified functions, compare errors
        if self.verified == value.verified:
            # Less errors is better
            if self.errors < value.errors:
                return True
            # If same number of errors, less Verus errors is better
            if self.errors == value.errors and self.verus_errors < value.verus_errors:
                return True

        return False

    def is_good_code_next_phase(self, value: object, abs_diff=2) -> bool:
        # Check whether self is a good code to value
        if not isinstance(value, EvalScore):
            return False

        # Compilation error is the highest priority - code that compiles is ALWAYS better
        if self.compilation_error != value.compilation_error:
            return not self.compilation_error

        # Allow a small reduction in verified functions when moving between phases
        # but compilation status must be preserved
        if self.verified >= value.verified - abs_diff:
            return True

        return False

    def __eq__(self, value: object) -> bool:
        if not isinstance(value, EvalScore):
            return False
        return (
            self.verified == value.verified
            and self.errors == value.errors
            and self.compilation_error == value.compilation_error
            and self.verus_errors == value.verus_errors
        )

    def __lt__(self, value: object) -> bool:
        if not isinstance(value, EvalScore):
            raise Exception("Invalid comparison")
        # Compilation error is the highest priority
        if self.compilation_error != value.compilation_error:
            return self.compilation_error
        # Then compare verified count
        if self.verified != value.verified:
            return self.verified < value.verified
        # Then compare error count
        if self.errors != value.errors:
            return self.errors > value.errors
        # Finally compare verus error count
        if self.verus_errors != value.verus_errors:
            return self.verus_errors > value.verus_errors
        return False

    def __gt__(self, value: object) -> bool:
        """
        Compare two EvalScore objects to determine if self is better than value.

        Args:
            value: Another EvalScore object to compare with

        Returns:
            True if self is better than value, False otherwise
        """
        # Handle edge cases in dummy mode
        if not isinstance(value, EvalScore):
            # Log but don't throw exception (prevents crashes in dummy mode)
            import logging

            logging.getLogger("EvalScore").warning(
                f"Attempted invalid comparison between EvalScore and {type(value)}"
            )
            return False

        # Compilation error is the highest priority - code that compiles is ALWAYS better
        if self.compilation_error != value.compilation_error:
            return not self.compilation_error

        # For code that both compile or both fail to compile, compare other metrics
        try:
            # Handle negative values safely (can happen in dummy mode)
            if self.verified != value.verified:
                return self.verified > value.verified
            if self.errors != value.errors:
                return self.errors < value.errors
            if self.verus_errors != value.verus_errors:
                return self.verus_errors < value.verus_errors
        except Exception as e:
            # If any comparison fails, log it and return False
            import logging

            logging.getLogger("EvalScore").warning(
                f"Error during score comparison: {e}"
            )
            return False

        return False

    def __le__(self, value: object) -> bool:
        return self < value or self == value

    def __ge__(self, value: object) -> bool:
        return self > value or self == value

    def __str__(self) -> str:
        return (
            f"Compilation Error: {self.compilation_error},"
            f" Verified: {self.verified},"
            f" Errors: {self.errors},"
            f" Verus Errors: {self.verus_errors}"
        )


# Don't use dummy mode by default
DUMMY_MODE = False


def fix_trivial_error(code: str) -> str:
    code = code.replace("/!", "//!")
    code = code.replace("!/", "//!")
    code = code.replace("//!", "//")
    return code


class VEval:
    def __init__(self, code: str, logger=None):
        self.logger = logger
        self.code = fix_trivial_error(code)
        # JSON reported by verus, does not include detailed erros(which is reported from rustc)
        self.verus_result = None
        # JSON reported by rustc, including any compliatoin errors and verus verification errors.
        # rustc reports multiple errors in multiple JSON objects to stderr.
        self.rustc_result = []
        # Parsed verus errors. Only verus exclusive errors(as listed in VerusErrorType) are parsed and stored. Compliation/sytanx/other errors are not stored.
        self.verus_errors = []
        self.verus_path = verus.verus_path
        self.compilation_error = False
        self.rustc_out = ""
        self.verus_out = ""

        # In dummy mode, we'll pretend to have basic compilation issues
        self.dummy_mode = DUMMY_MODE

        # Try to find a valid verus_path if ours is None
        if self.verus_path is None:
            # See if we can find verus in the environment
            import os

            verus_from_env = os.environ.get("VERUS_PATH")
            if verus_from_env and os.path.exists(verus_from_env):
                self.verus_path = verus_from_env
                if self.logger:
                    self.logger.info(
                        f"Found Verus path from environment: {self.verus_path}"
                    )
                # Update the global verus object too
                verus.set_verus_path(self.verus_path)
            elif os.environ.get("ENABLE_VEVAL", "1") == "1":
                # Specifically enabled but no path found - log a warning
                if self.logger:
                    self.logger.warning(
                        "VEval enabled but no verus_path found. Please set verus_path in config or VERUS_PATH environment variable."
                    )

        # Also set dummy mode if verus_path is None
        if self.verus_path is None:
            self.dummy_mode = True
            if self.logger:
                self.logger.warning(
                    "VEval in dummy mode (no verus_path). Will return placeholder results."
                )
        elif self.dummy_mode and self.logger:
            self.logger.warning("VEval in dummy mode. Will return placeholder results.")

    def eval_and_get_score(
        self, max_errs=5, json_mode=True, func_name=None
    ) -> EvalScore:
        self.eval(max_errs, json_mode, func_name)
        return self.get_score()

    def get_score(self) -> EvalScore:
        verified = self.get_verified()
        errors = self.get_errors()
        return EvalScore(
            verified, errors, self.compilation_error, len(self.verus_errors)
        )

    # Run verus on the code and parse the output.
    def eval(
        self,
        max_errs=5,
        json_mode=True,
        func_name=None,
        no_verify=False,
        log_dir=None,
        expand_errors=False,
    ) -> None:
        if self.dummy_mode:
            if self.logger:
                self.logger.warning(
                    "VEval in dummy mode. Generating placeholder results."
                )

            # Simulate a basic evaluation result
            self.verus_errors = ["Dummy error: TODO placeholder not implemented"]
            self.verus_out = "Dummy output: This is a simulation of Verus output"
            self.rustc_out = "error[E0999]: TODO placeholders need to be implemented"
            return

        with tempfile.NamedTemporaryFile(mode="w", delete=False) as f:
            f.write(self.code)
            code_path = f.name
        multiple_errors = f"--multiple-errors {max_errs}" if max_errs > 0 else ""
        err_format = "--output-json --error-format=json" if json_mode else ""
        # cmd = (f"{self.verus_path} {multiple_errors} {err_format} {code_path}").split(" ")
        # Bug fix: code_path may contain white space
        cmd = (f"{self.verus_path} {multiple_errors} {err_format}").split(" ")
        cmd += [code_path]
        if func_name:
            cmd += ["--verify-function", func_name, "--verify-root"]
        if no_verify:
            cmd += ["--no-verify"]
        if not (log_dir is None):
            # Add log to the default file log_dir if log_dir is not empty.
            # When this is enabled, verus will produce log, including:
            # - callgraph,
            # - verus intermediate language (vir),
            # - and smt file
            # Maybe useful for in-depth analysis
            if log_dir != "":
                cmd += ["--log-dir", log_dir]
            cmd += ["--log-all"]
        if expand_errors:
            # When expand_errors = true,
            # verus will report which postcond is established and which are not
            cmd += ["--expand-errors"]
        # self.logger.info(f"Running command: {' '.join(cmd)}")
        m = subprocess.run(cmd, capture_output=True, text=True)
        verus_out = m.stdout
        rustc_out = m.stderr
        os.unlink(code_path)

        self.verus_out = verus_out
        self.rustc_out = rustc_out

        if not json_mode:
            return

        try:
            self.verus_result = json.loads(self.verus_out)
        except json.JSONDecodeError as e:
            self.verus_result = None

        # If verus succeed, but rustc failed, then it is a compilation error.
        if self.verus_succeed() and m.returncode != 0:
            self.compilation_error = True

        for rust_err in rustc_out.split("\n")[:-1]:
            try:
                e = json.loads(rust_err)
            except json.JSONDecodeError as e:
                continue
            if not isinstance(e, dict):
                self.logger.error(f"Unexpected rust err output: {e}")
                continue
            self.rustc_result.append(e)
            if "level" in e and e["level"] == "error":
                if "message" in e and "aborting due to" in e["message"].lower():
                    continue  # Skip trivial aborting errors.
                # Make unclosed delimiter error worse than other errors
                if "unclosed delimiter" in e["message"]:
                    self.verus_errors.append(
                        VerusError({"message": "unclosed delimiter", "spans": []})
                    )
                self.verus_errors.append(VerusError(e))

    # Returns the number of verifed functions.
    def get_verified(self) -> int:
        if not self.verus_result:
            self.logger.error(f"Failure in VEval.get_verified. Rust Syntax error.")
        try:
            verified = self.verus_result["verification-results"]["verified"]
        except Exception as e:
            self.logger.error(
                f"Failure in VEval.get_verified. Verus Compilation error."
            )
            verified = -1
            self.compilation_error = True
        return verified

    # Returns the count of verified functions (convenience alias for get_verified)
    def get_verified_count(self) -> int:
        return self.get_verified()

    # Returns the number of failed functions.
    def get_errors(self) -> int:
        if not self.verus_result:
            self.logger.error(f"Failure in VEval.get_errors. Rust Syntax error.")
        try:
            errors = self.verus_result["verification-results"]["errors"]
        except Exception as e:
            self.logger.error(f"Failure in VEval.get_errors. Verus Compilation error.")
            errors = 999
            self.compilation_error = True
        return errors

    # Returns True if verus verification succeeded. If the compilation fails, verus also reports success as True,
    # but we consider it as a failure.
    def verus_succeed(self) -> bool:
        if not self.verus_result:
            Exception("No Verus result")
        return (
            self.compilation_error
            and self.verus_result["verification-results"]["success"]
        )

    def score(self) -> tuple[int, int]:
        return (self.get_verified(), self.get_errors())

    # Returns a list of ErrorTrace for PostCondFail errors
    def get_failed_postconds(self) -> list[ErrorTrace]:
        if not self.verus_result:
            Exception("No Verus result")

        if self.compilation_error:
            return []

        ret = []
        for e in self.verus_errors:
            if e.error == VerusErrorType.PostCondFail:
                for t in e.trace:
                    if t.label == VerusErrorLabel.FailedThisPostCond:
                        ret.append(t)
                        break

        return ret

    def get_failures(self, error_type: VerusErrorType = None) -> list[VerusError]:
        if not self.verus_result:
            Exception("No Verus result")

        # if self.compilation_error:
        #     return []

        ret = []
        for e in self.verus_errors:
            if error_type and e.error == error_type:
                ret.append(e)
            elif error_type is None:
                ret.append(e)
        return ret

    # Returns a list of VerusError if the error is from vstd
    def get_vstd_errors(self):
        if not self.verus_result:
            Exception("No Verus result")

        if self.compilation_error:
            return []

        ret = []
        for e in self.verus_errors:
            for t in e.trace:
                if t.is_vstd_err():
                    ret.append(e)
                    break

        return ret


if __name__ == "__main__":
    import argparse
    import sys

    from loguru import logger

    # Run simple EvalScore comparison tests
    def test_evalscore_comparison():
        print("Testing EvalScore comparison logic...")

        # Test that compilation status is prioritized correctly
        compiles_with_errors = EvalScore(
            verified=5, errors=3, compilation_error=False, verus_errors=5
        )
        noncompiles_with_better_metrics = EvalScore(
            verified=7, errors=1, compilation_error=True, verus_errors=1
        )

        # Compiling code should be BETTER than non-compiling code
        assert (
            compiles_with_errors > noncompiles_with_better_metrics
        ), "Prioritization error: Compilation status should be highest priority"

        # Non-compiling code should be WORSE than compiling code
        assert (
            noncompiles_with_better_metrics < compiles_with_errors
        ), "Prioritization error: Compilation status should be highest priority"

        # is_good_repair should return False when introducing compilation errors
        assert not noncompiles_with_better_metrics.is_good_repair(
            compiles_with_errors
        ), "Repair that introduces compilation errors should not be considered good"

        # is_good_code_next_phase should respect compilation status
        assert not noncompiles_with_better_metrics.is_good_code_next_phase(
            compiles_with_errors
        ), "Phase transition should respect compilation status"

        print("All tests passed!")

    # Simple argument parsing
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        test_evalscore_comparison()
        sys.exit(0)

    try:
        from utils import AttrDict
    except ImportError:

        class AttrDict(dict):
            def __getattr__(self, key):
                return self[key]

    # Parse arguments
    parser = argparse.ArgumentParser(description="Verus Copilot")
    parser.add_argument("--config", default="config.json", help="Path to config file")
    parser.add_argument("--mode", default="gen", help="Mode to run in (gen, refine)")
    parser.add_argument("--input", default="input.rs", help="Path to input file")
    parser.add_argument("--output", default="output.rs", help="Path to output file")
    parser.add_argument("--test", action="store_true", help="Run unit tests")
    args = parser.parse_args()

    if args.test:
        test_evalscore_comparison()
        sys.exit(0)

    # Check if config file exists
    if not os.path.isfile(args.config):
        print("Config file does not exist", file=sys.stderr)
        exit(1)

    config = json.load(open(args.config))
    config = AttrDict(config)
    verus.set_verus_path(config.verus_path)

    # In your main() function after resetting the config:
    reset_config("config-azure")
    # Add this line to set the verus path from the config:
    verus.set_verus_path(config.get("verus_path"))

    code = open(args.input).read()
    v = VEval(code, logger)
    print(
        f"Succeed: {v.verus_succeed()}, Verified: {v.get_verified()}, Errors: {v.get_errors()}"
    )
    print("Failed postconds:")
    for t in v.get_failed_postconds():
        print(t.get_text())
        print(t.get_lines())

    print("Failure from vstd:")
    for t in v.get_vstd_errors():
        print(t.get_text())
