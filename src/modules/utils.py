"""
Utility functions for VerusAgent modules.

This module provides shared functionality used across different inference and refinement modules,
particularly for writing, evaluating, and scoring code samples.
"""

from typing import List, Dict, Any, Optional, Tuple
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