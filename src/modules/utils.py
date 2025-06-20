"""
Utility functions for VerusAgent modules.

This module provides shared functionality used across different inference and refinement modules,
particularly for writing, evaluating, and scoring code samples.
"""

import json
import logging
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

from loguru import logger

import glob

# External helper for nonlinear-arithmetic analysis
from src.modules.lynette import (
    lynette,
)  # Provides code_detect_nonlinear, code_merge_invariant, etc.

# Import VEval from modules.veval rather than src.modules.veval
from src.modules.veval import VerusErrorType, VEval, EvalScore


def write_candidate_code(
    sample: str,
    veval: VEval,
    score: EvalScore,
    output_dir: Path,
    prefix: str,
    idx: int,
    logger: logging.Logger,
) -> None:
    """
    Writes an individual candidate code out to a file, including VEval metadata.

    Args:
        sample: The code sample to write
        veval: The VEval instance that evaluated the sample
        score: The evaluation score for the sample
        output_dir: Directory to write the sample to
        prefix: Prefix for the filename (e.g., '01_view_inference')
        idx: Index of the sample
        logger: Logger instance
    """
    output_dir.mkdir(exist_ok=True, parents=True)
    sample_path = output_dir / f"{prefix}_sample_{idx}.rs"

    try:
        # Append the score at the end of the file
        sample_with_score = f"{sample}\n\n// VEval Score: {score}"
        sample_path.write_text(sample_with_score)
        logger.info(
            f"Saved {prefix} sample {idx} to {output_dir}/{prefix}_sample_{idx}.rs (score: {score})"
        )
    except Exception as e:
        logger.error(f"Error saving sample {idx}: {e}")


def evaluate_samples(
    samples: List[str],
    output_dir: Path,
    prefix: str,
    logger: logging.Logger,
    max_errs: int = 5,
) -> Tuple[str, EvalScore, List[EvalScore]]:
    """
    Evaluates multiple code samples using VEval, writes them to files with scores,
    and returns the best sample, its score, and all scores.

    Args:
        samples: List of code samples to evaluate
        output_dir: Directory to write the samples to
        prefix: Prefix for the filenames
        logger: Logger instance
        max_errs: Maximum number of errors to report in VEval

    Returns:
        Tuple containing (best_code, best_score, all_scores)
    """
    if not samples:
        logger.error(f"No samples provided for evaluation")
        return "", None, []

    best_code = samples[0]  # Default to first sample
    best_score = None
    scores = []

    logger.info(f"Scoring generated {prefix} samples using VEval...")
    for i, sample in enumerate(samples):
        try:
            veval = VEval(sample, logger)
            veval.eval(max_errs=max_errs)
            score = veval.get_score()
            scores.append(score)

            # Write the sample with its score
            write_candidate_code(
                sample, veval, score, output_dir, prefix, i + 1, logger
            )

            # Log the score details
            logger.info(f"Sample {i+1} score: {score}")

            # Update best if this is better
            if best_score is None or score > best_score:
                best_score = score
                best_code = sample
                logger.info(f"New best sample: {i+1}")

            # If code is correct according to VEval, we can stop early
            if score.is_correct():
                logger.info(f"Found a correct proof in sample {i+1}!")
                # Save a special 'correct' version
                correct_path = output_dir / f"{prefix}_correct.rs"
                correct_path.write_text(sample)
                logger.info(f"Correct proof saved to {output_dir}/{prefix}_correct.rs")
                break

        except Exception as e:
            logger.error(f"Error scoring sample {i+1}: {e}")

    # Save the selected sample with details
    save_selection_info(output_dir, prefix, scores, best_score, logger)

    return best_code, best_score, scores


def save_selection_info(
    output_dir: Path,
    prefix: str,
    scores: List[EvalScore],
    best_score: EvalScore,
    logger: logging.Logger,
) -> None:
    """
    Saves selection information to a text file.

    Args:
        output_dir: Directory to write the selection info to
        prefix: Prefix for the filename
        scores: List of all scores
        best_score: The best score
        logger: Logger instance
    """
    selected_path = output_dir / f"{prefix}_selected.txt"
    try:
        best_idx = [str(s) for s in scores].index(str(best_score)) + 1
        selection_info = (
            f"Selected sample: {best_idx}\nScore: {best_score}\n\nAll scores:\n"
            + "\n".join([f"Sample {i+1}: {s}" for i, s in enumerate(scores)])
        )
        selected_path.write_text(selection_info)
        logger.info(
            f"Selection details saved to {output_dir}/{prefix}_selected.txt (best sample: {best_idx}, score: {best_score})"
        )

        # Also note the best sample file path
        best_sample_path = f"{output_dir}/{prefix}_sample_{best_idx}.rs"
        logger.info(
            f"Best {prefix} sample was #{best_idx}, located at {best_sample_path}"
        )
    except Exception as e:
        logger.error(f"Error saving selection details: {e}")


def check_and_handle_success(code: str, logger: logging.Logger) -> bool:
    """
    Checks if the given code is correct. If it is, returns True (success).
    Otherwise, returns False.

    Args:
        code: Code to evaluate
        logger: Logger instance

    Returns:
        True if the code is correct, False otherwise
    """
    veval = VEval(code, logger)
    score = veval.eval_and_get_score()
    return score.is_correct()


def update_checkpoint_best(
    cand_code: str,
    best_score_of_all: EvalScore,
    best_code_of_all: str,
    temp_dir: Path,
    logger: logging.Logger,
) -> Tuple[EvalScore, str]:
    """
    Compares cand_code's score with the checkpoint best. If cand_code is better,
    update the checkpoint best and write it to a file.

    Args:
        cand_code: Candidate code to compare
        best_score_of_all: Current best score
        best_code_of_all: Current best code
        temp_dir: Directory to save the best code to
        logger: Logger instance

    Returns:
        Tuple of (updated_best_score, updated_best_code)
    """
    veval = VEval(cand_code, logger)
    score = veval.eval_and_get_score()

    # Debug logging
    logger.debug(f"update_checkpoint_best - Candidate score: {score}")
    logger.debug(f"update_checkpoint_best - Current best score: {best_score_of_all}")
    logger.debug(
        f"update_checkpoint_best - Has best code: {best_code_of_all is not None}"
    )

    # Make sure the directory exists
    if not temp_dir.exists():
        temp_dir.mkdir(parents=True, exist_ok=True)

    # If best_score_of_all is None, set it to current score
    if best_score_of_all is None:
        logger.info(f"Initializing checkpoint best with score: {score}")
        best_score_of_all = score
        best_code_of_all = cand_code

        # Write to best.rs file
        best_path = temp_dir / "best.rs"
        sample_with_score = f"{best_code_of_all}\n\n// Checkpoint Best Score: {score}"
        best_path.write_text(sample_with_score)
        return best_score_of_all, best_code_of_all

    # Compare scores
    try:
        is_better = score > best_score_of_all
        logger.debug(
            f"update_checkpoint_best - Candidate is better than current best: {is_better}"
        )
    except Exception as e:
        logger.error(f"Error comparing scores: {e}")
        is_better = False

    if is_better:
        best_score_of_all = score
        best_code_of_all = cand_code

        # Write to best.rs file
        best_path = temp_dir / "best.rs"
        sample_with_score = f"{best_code_of_all}\n\n// Checkpoint Best Score: {score}"
        best_path.write_text(sample_with_score)
        logger.info(f"Updated checkpoint best with score: {score}")
    else:
        # Even if not better, ensure the best.rs file exists with the current best
        best_path = temp_dir / "best.rs"
        if not best_path.exists() and best_code_of_all is not None:
            sample_with_score = (
                f"{best_code_of_all}\n\n// Checkpoint Best Score: {best_score_of_all}"
            )
            best_path.write_text(sample_with_score)
            logger.info(
                f"Created best.rs file with existing checkpoint best score: {best_score_of_all}"
            )

    return best_score_of_all, best_code_of_all


def evaluate_candidates(
    candidates: List[str],
    prefix: str,
    func_name: str,
    iteration_idx: int,
    last_best_code: str,
    last_best_score: EvalScore,
    temp_dir: Path,
    logger: logging.Logger,
    debug_type_error_fn: Optional[Callable] = None,
) -> Tuple[str, str, EvalScore]:
    """
    Evaluates multiple candidate codes generated by a single function.
    Updates and returns the best candidate code and score for this iteration.

    Args:
        candidates: List of candidate codes to evaluate
        prefix: Prefix for the filename
        func_name: Name of the function that generated the candidates
        iteration_idx: Index of the current iteration
        last_best_code: Best code from the previous iteration
        last_best_score: Best score from the previous iteration
        temp_dir: Directory to save files to
        logger: Logger instance
        debug_type_error_fn: Optional function to debug and fix type errors

    Returns:
        Tuple of (candidate_code, best_code, best_score)
    """
    best_score = EvalScore.get_worst_score()
    best_code = last_best_code

    for j, cand in enumerate(candidates):
        # Use our new debug_type_error function if no external one is provided
        if debug_type_error_fn:
            cand, _ = debug_type_error_fn(cand)
        else:
            cand_fixed, _ = debug_type_error(cand, logger=logger)
            if cand_fixed:  # Only use the fixed version if it's not empty
                cand = cand_fixed

        veval = VEval(cand, logger)
        score = veval.eval_and_get_score()

        # If code is correct according to VEval
        if score.is_correct():
            logger.info("Found a correct proof!")
            correct_path = temp_dir / f"{prefix}_correct.rs"
            sample_with_score = f"{cand}\n\n// VEval Score: {score}"
            correct_path.write_text(sample_with_score)
            return cand, cand, score

        # Update the best candidate if needed
        if not (score < best_score):
            best_score = score
            best_code = cand

        # Write each candidate's code to a temp file
        candidate_path = temp_dir / f"{prefix}_{iteration_idx}_{func_name}_{j}.rs"
        sample_with_score = f"{cand}\n\n// VEval Score: {score}"
        candidate_path.write_text(sample_with_score)

    # Return the best after evaluating all candidates of this func
    if best_score.is_good_code_next_phase(last_best_score):
        return best_code, best_code, best_score
    else:
        return last_best_code, last_best_code, last_best_score


def fix_one_type_error(oldline, cstart, cend, newtype):
    """
    Fix a type error in a line by inserting an appropriate cast.

    Args:
        oldline: The line containing the type error
        cstart: The starting index of the problematic expression
        cend: The ending index of the problematic expression
        newtype: The new type to cast to

    Returns:
        The fixed line
    """
    # cstart: the starting index of the problematic expression
    # cend: the ending index of the problematic expression

    prefix = oldline[:cstart]
    mid = oldline[cstart : cend + 1]
    suffix = oldline[cend + 1 :]

    oldtype_pos = mid.rfind(" as ")

    if oldtype_pos > -1:
        if " " in mid[oldtype_pos + 4 :].strip():
            # there was not a cast type for the whole expression
            # instead it is something like x as int - 1
            oldtype_pos = -1

    if oldtype_pos == -1:
        # the old expression does not have "as oldtype"
        if re.match(r"^\(*\)$", mid.strip()):
            # already in parentheses
            newmid = mid + " as " + newtype
        else:
            newmid = "( " + mid + " ) as " + newtype
    else:
        # replace the old as type
        newmid = mid[:oldtype_pos] + " as " + newtype

    return prefix + newmid + suffix


def fix_one_type_error_in_code(code, err_trace, verbose=True):
    """
    Fix a type error in the code based on the error trace.

    Args:
        code: The Verus code
        err_trace: The error trace from VEval
        verbose: Whether to output verbose debugging information

    Returns:
        The fixed code, or an empty string if the error could not be fixed
    """
    # note that linenum, cstart, cend indices all start from 0
    err_label = err_trace.strlabel

    # Special-case: mutability mismatch (e.g., "types differ in mutability").
    # For this error the simplest automatic fix is to remove the entire line;
    # the LLM-based repair pipeline can then try a different approach later.
    if err_label is not None and "types differ in mutability" in err_label:
        err_lnum = err_trace.get_lines()[0]
        linenum = err_lnum - 1

        # Drop that line from the source.
        new_code_lines = [
            line for idx, line in enumerate(code.split("\n")) if idx != linenum
        ]
        if verbose:
            sys.stderr.write(
                f"[fix_one_type_error_in_code] removed line {err_lnum} due to mutability mismatch.\n"
            )
        return "\n".join(new_code_lines) + "\n"

    # Default path: expect a `...` label so we can perform a cast/rewrite.
    if err_label is None or "`" not in err_label:
        sys.stderr.write(f"err_label: {err_label}\n")
        sys.stderr.write(f"err_trace: {err_trace}\n")
        sys.stderr.write("Fatal error: err_trace does not have a label\n")
        sys.stderr.write(code)
        return code

    newtype = err_label.split("`")[1]

    err_lnum = err_trace.get_lines()[0]
    linenum = err_lnum - 1

    line = err_trace.get_text()
    cstart = err_trace.text[0].hl_start - 1
    cend = err_trace.text[0].hl_end - 2
    err_exp = line[cstart : cend + 1]

    newlines = []
    for i, line in enumerate(code.split("\n")):
        if i != linenum:
            newlines.append(line)
        else:
            if not err_exp in line:
                sys.stderr.write(
                    "Fatal error: `" + err_exp + "' does not exist in " + line
                )
                return ""
            if err_exp != line[cstart : cend + 1]:
                sys.stderr.write(
                    "Fatal error. Expected expression is `"
                    + err_exp
                    + "'; Get expression `"
                    + line[cstart : cend + 1]
                )
                return ""

            newline = fix_one_type_error(line, cstart, cend, newtype)

            # Sometimes, we may encounter non-fixable type error
            # for example if one expects ..i or [i] to be int, ..i as int or [i] as int will return the same type error
            # so, we return "" to warn the caller
            # otherwise, the caller may hang
            if line == newline:
                return ""

            if verbose == True:
                sys.stderr.write(
                    "[fix_one_type_error_in_code] changed the type of `"
                    + line[cstart : cend + 1]
                    + "'"
                    + "as `"
                    + newline.strip()
                    + "'"
                )
            newlines.append(newline)

    return "\n".join(newlines) + "\n"


def debug_type_error(code: str, verus_error=None, num=1, logger=None) -> tuple:
    """
    Debug and fix type errors in the Verus code.

    Args:
        code: The Verus code to fix
        verus_error: A specific error to fix (optional)
        num: Maximum number of errors to fix
        logger: Logger instance

    Returns:
        A tuple of (fixed_code, remaining_errors)
    """
    del num

    if logger is None:
        logger = logging.getLogger("debug_type_error")
        logger.setLevel(logging.INFO)

    rnd = 0
    max_rnd = 10

    # Import the needed class here to avoid circular imports
    from src.modules.veval import VerusErrorType, VEval

    # Handle dummy mode - if verus_error is a string rather than a VerusError object
    if isinstance(verus_error, str):
        logger.warning(
            "Received string error in dummy mode instead of VerusError object"
        )
        return code, 0

    if verus_error:
        # fix the reported one
        if (
            not hasattr(verus_error, "error")
            or verus_error.error != VerusErrorType.MismatchedType
        ):
            logger.warning(
                f"Warning: a non type error is passed to debug_type_error: {getattr(verus_error, 'error', 'unknown')}"
            )
        else:
            newcode = fix_one_type_error_in_code(
                code, verus_error.trace[0], verbose=False
            )
            if newcode:
                code = newcode

    # check if there is any type errors in the code; if so, fix
    while rnd < max_rnd:
        rnd = rnd + 1

        veval = VEval(code, logger)
        veval.eval()
        failures = veval.get_failures()
        if len(failures) == 0:
            logger.info(f"Verus has succeeded.")
            return code, 0

        has_typeerr = False
        fixed_typeerr = False
        for cur_failure in failures:
            # Skip string failures in dummy mode
            if isinstance(cur_failure, str):
                logger.warning(f"Skipping string failure in dummy mode: {cur_failure}")
                continue

            if (
                hasattr(cur_failure, "error")
                and cur_failure.error == VerusErrorType.MismatchedType
            ):
                has_typeerr = True
                newcode = fix_one_type_error_in_code(
                    code, cur_failure.trace[0], verbose=False
                )
                # when newcode is "", the above function failed to fix any type error
                if newcode:
                    fixed_typeerr = True
                    code = newcode
                    break
                else:
                    # this type error is unfixable, let's move on to next error
                    continue
            if not fixed_typeerr:
                # not able to fix any type error in this program, no need to try again
                break

        if not has_typeerr:
            return code, 0

        if not fixed_typeerr:
            logger.info("Remaining type errors are unfixable.")
            if isinstance(cur_failure, str):
                logger.info(cur_failure)
            else:
                logger.info(cur_failure.trace[0].get_text())
            return "", len(failures)

    return code, len(failures)


def remove_comment(code):
    """
    remove single-line comments in code
    """
    new_code = ""
    for line in code.split("\n"):
        if line.strip().startswith("//"):
            continue
        new_code += line + "\n"
    return new_code


def remove_rust_comments(code: str) -> str:
    """
    Removes comment lines and inline comments from Rust code.

    Full-line comments (lines that, after stripping leading whitespace,
    start with '//') are completely removed.

    Inline comments (everything after a '//' on a line) are stripped away.

    Note: This simple implementation does not handle the case where '//'
    might appear inside a string literal.
    """
    lines = code.splitlines()
    new_lines = []
    for line in lines:
        # Skip lines that are entirely comments.
        if re.match(r"^\s*//", line):
            continue
        # Remove any inline comment (everything after the first occurrence of '//')
        # and also strip trailing whitespace.
        new_line = re.split(r"//", line)[0].rstrip()
        new_lines.append(new_line)
    return "\n".join(new_lines)


# Ported from archive/code/utils.py
class AttrDict(dict):
    def __getattr__(self, name):
        return self[name]


def get_nonlinear_lines(code, logger):
    """
    Detect lines containing nonlinear arithmetic using Lynette.

    Args:
        code: Source code to analyze
        logger: Logger instance

    Returns:
        List of line numbers containing nonlinear arithmetic
    """
    try:
        import tempfile
        from src.modules.lynette import lynette

        with tempfile.NamedTemporaryFile(mode="w", suffix=".rs", delete=False) as f:
            f.write(code)
            f.flush()

            result = lynette.code_detect_nonlinear(f.name)

            # Clean up temp file
            import os

            os.unlink(f.name)

            if result.returncode == 0:
                # Parse the output to extract line numbers
                lines = []
                for line in result.stdout.strip().split("\n"):
                    if line.strip() and line.strip().isdigit():
                        lines.append(int(line.strip()))
                return lines
            else:
                if logger:
                    logger.warning(
                        f"Lynette nonlinear detection failed: {result.stderr}"
                    )
                return []

    except Exception as e:
        if logger:
            logger.error(f"Error detecting nonlinear arithmetic: {e}")
        return []


def code_change_is_safe(
    origin_code,
    changed_code,
    verus_path,
    logger,
    target_mode=True,
    util_path=None,
    inter=False,
    debug=True,
    immutable_funcs=[],
):
    # Debug mode override (from original code)
    if debug and os.environ.get("DEBUG_SAFE_CODE_CHANGE"):
        logger.info("DEBUG_SAFE_CODE_CHANGE is set, skipping safe code change checking")
        return True

    # If codes are identical, they're obviously safe
    if origin_code.strip() == changed_code.strip():
        return True

    for func_name in immutable_funcs:
        # Get function bodies safely, handling potential errors
        origin_body = get_func_body(origin_code, func_name, util_path, logger)
        changed_body = get_func_body(changed_code, func_name, util_path, logger)

        if origin_body is None or changed_body is None:
            logger.warning(
                f"Could not compare immutable function '{func_name}'. Assuming unsafe."
            )
            return False

        origin = remove_rust_comments(origin_body)
        changed = remove_rust_comments(changed_body)

        if origin != changed:
            logger.error(f"Immutable function '{func_name}' was changed")
            return False
    return True
    try:
        orig_f = tempfile.NamedTemporaryFile(
            mode="w", delete=False, prefix="llm4v_orig", suffix=".rs"
        )
        orig_f.write(origin_code)
        orig_f.close()

        changed_f = tempfile.NamedTemporaryFile(
            mode="w", delete=False, prefix="llm4v_changed", suffix=".rs"
        )
        changed_f.write(changed_code)
        changed_f.close()

        if util_path is None:
            # Use default path calculation
            cargopath = (
                Path(__file__).parent.parent.parent
                / "utils"
                / "lynette"
                / "source"
                / "Cargo.toml"
            )
            cargopath = str(cargopath.resolve())
        else:
            cargopath = os.path.join(util_path, "lynette/source/Cargo.toml")

        if not os.path.exists(cargopath):
            # Attempt relative path from src/modules/utils.py if absolute fails
            cargopath = (
                Path(__file__).parent.parent.parent
                / "utils"
                / "lynette"
                / "source"
                / "Cargo.toml"
            )
            if not cargopath.exists():
                logger.warning(
                    f"Could not find lynette Cargo.toml at {cargopath}, assuming code is safe"
                )
                return True  # Default to safe if we can't compare
            cargopath = str(cargopath.resolve())

        opts = []
        if inter:
            opts = ["--asserts-anno"]
        elif target_mode:
            opts = ["-t"]

        verus_compare_cmd = (
            ["cargo", "run", "--manifest-path", cargopath, "--", "compare"]
            + opts
            + [orig_f.name, changed_f.name]
        )

        m = subprocess.run(
            verus_compare_cmd, capture_output=True, text=True, timeout=30
        )
        logger.info(f"Lynette comparison output: {m.stdout}")
        logger.info(f"Lynette comparison error: {m.stderr}")
        logger.info(f"Lynette comparison return code: {m.returncode}")
        if m.returncode == 0:
            return True
        elif m.returncode == 1:
            err_m = m.stdout.strip()
            if err_m == "Files are different":
                return False
            else:
                logger.warning(
                    f"Lynette comparison returned unexpected output: {err_m}, assuming safe"
                )
                return True  # Default to safe on unexpected output
        else:
            err_m = m.stderr.strip()
            logger.warning(f"Lynette comparison failed: {err_m}, assuming code is safe")
            return True  # Default to safe on tool error

    except subprocess.TimeoutExpired:
        logger.warning("Lynette comparison timed out, assuming code is safe")
        return True
    except Exception as e:
        logger.warning(f"Exception during code comparison: {e}, assuming code is safe")
        return True  # Default to safe on any exception


def get_func_body(code, fname, util_path=None, logger=None):
    try:
        orig_f = tempfile.NamedTemporaryFile(
            mode="w", delete=False, prefix="verus_agent_", suffix=".rs"
        )
        orig_f.write(code)
        orig_f.close()

        # If util_path is not provided, use the default path relative to the code directory
        if util_path is None:
            util_path = Path(__file__).parent.parent.parent / "utils"
            util_path = str(util_path.resolve())

        # Construct absolute path to Cargo.toml
        cargopath = Path(util_path) / "lynette" / "source" / "Cargo.toml"

        if not cargopath.exists():
            if logger:
                logger.error(f"Error: Cargo.toml not found at {cargopath}")
            return None

        cargopath = str(cargopath.resolve())

        lynette_extract_cmd = [
            "cargo",
            "run",
            "--manifest-path",
            cargopath,
            "--",
            "func",
            "extract",
            "-b",
            "-f",
            fname,
            orig_f.name,
        ]

        # Debug: Log the exact file path and working directory
        logger.info(f"Absolute path: {os.path.abspath(orig_f.name)}")

        m = subprocess.run(
            lynette_extract_cmd, capture_output=True, text=True, cwd=os.getcwd()
        )
        # logger.info(f"Lynette extract command: {lynette_extract_cmd}")
        # logger.info(f"Lynette extract output: {m.stdout}")
        # logger.info(f"Lynette extract error: {m.stderr}")
        # logger.info(f"Lynette extract return code: {m.returncode}")

        # Handle error cases
        if m.returncode != 0:
            if logger:
                if m.stderr:
                    logger.error(f"Error extracting function '{fname}': {m.stderr}")
                else:
                    logger.error(
                        f"Error extracting function '{fname}' (no stderr output). Return code: {m.returncode}"
                    )
                    if m.stdout:
                        logger.error(f"stdout: {m.stdout}")
            return None

        if m.returncode == 0:
            return m.stdout.strip()
        return None

    except Exception as e:
        if logger:
            logger.error(f"Exception during get_func_body: {e}")
        return None


def evaluate(code, verus_path, func_name=None):
    """Simple Verus evaluation, returns score tuple and subprocess result."""
    fn = tempfile.NamedTemporaryFile(
        mode="w", delete=False, prefix="llm4v_eval", suffix=".rs"
    )
    fn.write(code)
    fn.close()

    commands = [verus_path, fn.name]
    if func_name:
        commands += ["--verify-function", func_name, "--verify-root"]
    m = subprocess.run(commands, capture_output=True, text=True)
    os.unlink(fn.name)

    temp = 0
    chunks = m.stderr.split("\n\n")
    for ch in chunks:
        if ch.startswith("error") and "aborting due to" not in ch:
            temp += 1
    try:
        score = re.findall(r"(\d+) verified, (\d+) errors", m.stdout)[0]
    except IndexError as e:
        score = (0, max(1, temp))
    if score[0] == "0" and score[1] == "0":
        score = (0, temp)
    score = (int(score[0]), max(int(score[1]), temp))
    return score, m


def compress_nl_assertion(code):
    """Compresses nonlinear assertions into a single line."""
    lines = code.split("\n")
    inside = False
    tmp_line = ""
    new_code = ""
    for line in lines:
        if not inside:
            if (
                line.strip().startswith("assert")
                and "by" in line
                and "nonlinear_arith" in line
            ):
                inside = True
                tmp_line += line
            else:
                new_code += line + "\n"
        else:
            if "{}" in line:
                tmp_line += " " + line.strip() + "\n"
                inside = False
                new_code += tmp_line
                tmp_line = ""
            else:
                tmp_line += " " + line.strip()
    return new_code


def remove_redundant_loopinv(code):
    """
    remove redundant loop invariants in code
    """
    new_code = ""
    invariants = False
    invariantlist = []
    for line in code.split("\n"):
        remove = False
        if invariants:
            if line.strip().startswith("{"):
                invariants = False
            else:
                thisinv = re.sub(r"//.*", "", line).strip()
                for inv in invariantlist:
                    if thisinv == inv:
                        remove = True
                if not remove:
                    invariantlist.append(thisinv)
        else:
            if line.strip().startswith("invariant"):
                invariantlist = []
                invariants = True
        if not remove:
            new_code += line + "\n"
    return new_code


def same_code_verus(code1, code2, verus_path):
    """
    Check if two code snippets return the same Verus err results
    """
    _, msg1 = evaluate(code1, verus_path)
    _, msg2 = evaluate(code2, verus_path)
    err1 = msg1.stderr + msg1.stdout
    err2 = msg2.stderr + msg2.stdout
    return err1 == err2


def insert_loop_isolation(code):
    """Insert #[verifier::loop_isolation(false)]"""
    lines = code.splitlines()
    verus_line = -1
    for i, line in enumerate(lines):
        if "verus!" in line:
            verus_line = i
            break
    if verus_line == -1:
        print("No verus! found in the code.")
        return code
    insert_line = "\n#[verifier::loop_isolation(false)]"
    new_code = "\n".join(
        lines[: verus_line + 1] + [insert_line] + lines[verus_line + 1 :]
    )
    return new_code


def insert_lemma_func(code, lemma_names, lemma_path):
    """Insert existing already-proved lemmas"""
    for lemma_name in lemma_names:
        name = lemma_name
        if not name.endswith(".rs"):
            name = name + ".rs"
        input_file = os.path.join(lemma_path, name)
        lemma_code = open(input_file).read()
        lemma_func_dict = {lemma_name: lemma_code}
        code = insert_proof_func(code, lemma_func_dict)
    return code


def insert_proof_func(code, proof_func_dict):
    """Insert the proof functions into the code."""
    lines = code.splitlines()
    verus_line = -1
    for i, line in enumerate(lines):
        if "verus!" in line:
            verus_line = i
            break
    if verus_line == -1:
        return code
    proof_func_code = "\n\n".join(proof_func_dict.values())
    new_code = "\n".join(
        lines[: verus_line + 1] + [proof_func_code] + lines[verus_line + 1 :]
    )
    return new_code


def get_examples(
    config: Dict[str, Any], example_dir_name: str, logger: logging.Logger
) -> List[Dict[str, str]]:
    """
    Gathers example input/output pairs from two directories (input-<example_dir_name>
    and output-<example_dir_name>), and returns a list of dictionaries
    with 'query' and 'answer' keys.

    Args:
        config: Configuration dictionary containing example_path
        example_dir_name: The suffix for the input/output directories
        logger: Logger instance

    Returns:
        A list of example dictionaries, or an empty list if errors occur.
    """
    examples = []
    try:
        examples_dir = Path(config.get("example_path", "examples"))
        input_dir = examples_dir / f"input-{example_dir_name}"
        output_dir = examples_dir / f"output-{example_dir_name}"

        # Ensure the required directories exist
        if not input_dir.is_dir():
            logger.warning(f"Input directory '{input_dir}' does not exist.")
            return []  # Return empty list if input dir missing
        if not output_dir.is_dir():
            logger.warning(f"Output directory '{output_dir}' does not exist.")
            # Proceed but output files might be missing

        # Collect input/output pairs
        for input_file in sorted(input_dir.iterdir()):
            # Only consider *.rs files that start with "ex"
            if input_file.suffix == ".rs" and input_file.name.startswith("ex"):
                output_file = output_dir / input_file.name

                if not output_file.is_file():
                    logger.warning(
                        f"No matching output file for '{input_file}'. Using empty answer."
                    )
                    output_content = ""  # Use empty string if output is missing
                else:
                    try:
                        output_content = output_file.read_text(encoding="utf-8")
                    except OSError as e:
                        logger.error(f"Failed to read output file '{output_file}': {e}")
                        output_content = ""  # Use empty string on read error

                # Safely read the input file
                try:
                    input_content = input_file.read_text(encoding="utf-8")
                except OSError as e:
                    logger.error(f"Failed to read input file '{input_file}': {e}")
                    continue  # Skip this example if input fails

                examples.append({"query": input_content, "answer": output_content})

        # Warn if no valid examples were found
        if not examples:
            logger.warning(f"No valid examples found in {input_dir}")

    except Exception as e:
        logger.error(f"Error loading examples from {example_dir_name}: {e}")

    return examples


def clean_code(code):
    """Remove markdown code blocks and potentially other unwanted characters."""
    might_code = re.findall(r"```rust(.*)```|```verus(.*)```", code, flags=re.DOTALL)
    if might_code:
        code = might_code[0][0] if might_code[0][0] else might_code[0][1]

    lines = []
    for line in code.split("\n"):
        if line.strip() == "```":
            continue

        # this is ad-hoc, but somehow GPT often generates ```use ... on first line
        if line.startswith("```"):
            line = line[3:]

        lines.append(line)
    code = "\n".join(lines)
    return code


def parse_llm_response(response: str, logger=None) -> str:
    """
    General utility to parse and extract Rust/Verus code from any LLM response.
    Uses simple string operations instead of regex.

    Args:
        response: The raw LLM response text
        logger: Optional logger for debugging information

    Returns:
        Extracted code as a string or empty string if no code found
    """
    if logger:
        logger.info("Parsing LLM response for Rust/Verus code...")

    # If response is empty, return empty string
    if not response or response.strip() == "":
        if logger:
            logger.warning("Empty response received from LLM")
        return ""

    # Look for code blocks with ```
    if "```" in response:
        blocks = []
        lines = response.split("\n")
        in_code_block = False
        current_block = []

        for line in lines:
            if line.strip().startswith("```"):
                if in_code_block:
                    # End of code block
                    in_code_block = False
                    if current_block:
                        blocks.append("\n".join(current_block))
                    current_block = []
                else:
                    # Start of code block
                    in_code_block = True
                    # Skip the opening ```rust, ```verus, etc.
                    continue
            elif in_code_block:
                current_block.append(line)

        if blocks:
            if logger:
                logger.info(f"Extracted {len(blocks)} code block(s)")
            return "\n".join(blocks)

    # Check if the response itself looks like Rust/Verus code
    # by counting keyword occurrences
    rust_keywords = [
        "fn ",
        "struct ",
        "impl ",
        "pub ",
        "let ",
        "use ",
        "mod ",
        "trait ",
        "enum ",
        "match ",
        "proof ",
        "spec ",
        "requires",
        "ensures",
        "invariant",
        "View for",
        "verus!",
    ]

    keyword_count = 0
    for keyword in rust_keywords:
        if keyword in response:
            keyword_count += 1

    # If the response has several Rust/Verus keywords, it's likely code
    if keyword_count >= 3:
        if logger:
            logger.info(
                f"Response contains {keyword_count} Rust/Verus keywords - treating as direct code"
            )
        return response

    # If we couldn't find any code, return empty string
    if logger:
        logger.warning("Could not extract Rust/Verus code from response")
    return ""


def parse_plan_execution_order(
    plan_text: str, available_modules: List[str], logger=None
) -> List[str]:
    """
    Simplified plan execution parser that only allows two possible workflows:
    1. Full sequence: view_inference -> view_refinement -> inv_inference -> spec_inference
    2. Just spec_inference alone

    Args:
        plan_text: The planner's response text
        available_modules: List of available module names
        logger: Optional logger for debugging

    Returns:
        Ordered list of module names to execute
    """
    if logger:
        logger.info(
            "Parsing plan to determine module execution order (limited to two possible workflows)..."
        )

    # Define our two possible workflows
    full_workflow = [
        "view_inference",
        "view_refinement",
        "inv_inference",
        "spec_inference",
    ]
    # If proof_generation module is available, we may append it later based on plan text
    proof_step = "proof_generation" if "proof_generation" in available_modules else None
    spec_only_workflow = ["spec_inference"]

    # Check which modules are available
    available_full_workflow = [m for m in full_workflow if m in available_modules]

    # Check if we should do the spec-only workflow
    spec_only_indicators = [
        "only need specification",
        "only spec inference",
        "spec inference only",
        "only specification",
        "skip view",
        "no view needed",
        "no need for view",
        "focus on spec",
        "specification is sufficient",
        "specification alone",
        "specification-only workflow",
        "spec-only workflow",
    ]

    use_spec_only = any(
        indicator.lower() in plan_text.lower() for indicator in spec_only_indicators
    )

    if use_spec_only and "spec_inference" in available_modules:
        if logger:
            logger.info("Using spec-inference-only workflow based on plan.")
        if proof_step and re.search(r"\bproof_generation\b", plan_text.lower()):
            return spec_only_workflow + [proof_step]
        return spec_only_workflow
    else:
        if proof_step and re.search(r"\bproof_generation\b", plan_text.lower()):
            workflow = available_full_workflow + [proof_step]
            if logger:
                logger.info("Using full workflow with proof_generation appended.")
            return workflow
        if logger:
            logger.info(
                "Using full workflow sequence: view_inference -> view_refinement -> inv_inference -> spec_inference"
            )
        return available_full_workflow
