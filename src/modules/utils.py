"""
Utility functions for VerusAgent modules.

This module provides shared functionality used across different inference and refinement modules,
particularly for writing, evaluating, and scoring code samples.
"""

from typing import List, Dict, Any, Optional, Tuple, Callable
from pathlib import Path
import os
import logging
import re
import sys

from modules.veval import VEval, EvalScore

def write_candidate_code(sample: str, veval: VEval, score: EvalScore, 
                         output_dir: Path, prefix: str, idx: int, 
                         logger: logging.Logger) -> None:
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
        logger.info(f"Saved {prefix} sample {idx} to {sample_path}")
    except Exception as e:
        logger.error(f"Error saving sample {idx}: {e}")

def evaluate_samples(samples: List[str], output_dir: Path, prefix: str, 
                     logger: logging.Logger, 
                     max_errs: int = 5) -> Tuple[str, EvalScore, List[EvalScore]]:
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
            write_candidate_code(sample, veval, score, output_dir, prefix, i+1, logger)
            
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
                break
                
        except Exception as e:
            logger.error(f"Error scoring sample {i+1}: {e}")
    
    # Save the selected sample with details
    save_selection_info(output_dir, prefix, scores, best_score, logger)
    
    return best_code, best_score, scores

def save_selection_info(output_dir: Path, prefix: str, scores: List[EvalScore], 
                        best_score: EvalScore, logger: logging.Logger) -> None:
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
        selection_info = f"Selected sample: {best_idx}\nScore: {best_score}\n\nAll scores:\n" + "\n".join([f"Sample {i+1}: {s}" for i, s in enumerate(scores)])
        selected_path.write_text(selection_info)
        logger.info(f"Selection details saved to {selected_path}")
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

def update_global_best(cand_code: str, best_score_of_all: EvalScore, 
                       best_code_of_all: str, temp_dir: Path, 
                       logger: logging.Logger) -> Tuple[EvalScore, str]:
    """
    Compares cand_code's score with the global best. If cand_code is better,
    update the global best and write it to a file.
    
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
    logger.debug(f"update_global_best - Candidate score: {score}")
    logger.debug(f"update_global_best - Current best score: {best_score_of_all}")
    logger.debug(f"update_global_best - Has best code: {best_code_of_all is not None}")
    
    # Make sure the directory exists
    if not temp_dir.exists():
        temp_dir.mkdir(parents=True, exist_ok=True)
    
    # If best_score_of_all is None, set it to current score
    if best_score_of_all is None:
        logger.info(f"Initializing global best with score: {score}")
        best_score_of_all = score
        best_code_of_all = cand_code
        
        # Write to best.rs file
        best_path = temp_dir / "best.rs"
        sample_with_score = f"{best_code_of_all}\n\n// VEval Score: {score}"
        best_path.write_text(sample_with_score)
        return best_score_of_all, best_code_of_all

    # Compare scores
    try:
        is_better = score > best_score_of_all
        logger.debug(f"update_global_best - Candidate is better than current best: {is_better}")
    except Exception as e:
        logger.error(f"Error comparing scores: {e}")
        is_better = False

    if is_better:
        best_score_of_all = score
        best_code_of_all = cand_code
        
        # Write to best.rs file
        best_path = temp_dir / "best.rs"
        sample_with_score = f"{best_code_of_all}\n\n// VEval Score: {score}"
        best_path.write_text(sample_with_score)
        logger.info(f"Updated global best with score: {score}")
    else:
        # Even if not better, ensure the best.rs file exists with the current best
        best_path = temp_dir / "best.rs"
        if not best_path.exists() and best_code_of_all is not None:
            sample_with_score = f"{best_code_of_all}\n\n// VEval Score: {best_score_of_all}"
            best_path.write_text(sample_with_score)
            logger.info(f"Created best.rs file with existing best score: {best_score_of_all}")
    
    return best_score_of_all, best_code_of_all

def evaluate_candidates(candidates: List[str], prefix: str, 
                        func_name: str, iteration_idx: int,
                        last_best_code: str, last_best_score: EvalScore, 
                        temp_dir: Path, logger: logging.Logger, 
                        debug_type_error_fn: Optional[Callable] = None) -> Tuple[str, str, EvalScore]:
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
    if err_label is None or not "`" in err_label:
        sys.stderr.write("Fatal error: err_trace does not have a label")
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
        logger = logging.getLogger('debug_type_error')
        logger.setLevel(logging.INFO)
    
    rnd = 0
    max_rnd = 10
    
    # Import the needed class here to avoid circular imports
    from modules.veval import VEval, VerusErrorType

    if verus_error:
        # fix the reported one
        if verus_error.error != VerusErrorType.MismatchedType:
            logger.warning(f"Warning: a non type error is passed to debug_type_error: {verus_error.error}")
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
            if cur_failure.error == VerusErrorType.MismatchedType:
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
            logger.info(cur_failure.trace[0].get_text())
            return "", len(failures)

    return code, len(failures) 