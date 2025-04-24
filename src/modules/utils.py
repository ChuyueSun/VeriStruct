"""
Utility functions for VerusAgent modules.

This module provides shared functionality used across different inference and refinement modules,
particularly for writing, evaluating, and scoring code samples.
"""

from typing import List, Dict, Any, Optional, Tuple, Callable
from pathlib import Path
import os
import logging

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
        if debug_type_error_fn:
            cand, _ = debug_type_error_fn(cand)

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