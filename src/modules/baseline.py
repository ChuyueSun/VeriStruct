"""
Baseline module for single-shot specification and proof generation.

This module provides a baseline approach that asks the LLM to generate both
specifications and proofs in a single call, without the multi-stage pipeline.
"""

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Dict, List

from src.infer import LLM
from src.modules.base import BaseModule
from src.modules.utils import (
    code_change_is_safe,
    evaluate_samples,
    get_examples,
    parse_llm_response,
    update_checkpoint_best,
)
from src.modules.veval import VEval
from src.utils.path_utils import prompt_dir, samples_dir


class BaselineModule(BaseModule):
    """
    Baseline module that generates specifications and proofs in a single LLM call.

    This serves as a baseline comparison against the multi-stage pipeline approach.
    """

    def __init__(self, config, logger, immutable_funcs=None):
        super().__init__(
            name="baseline",
            desc="Single-shot specification and proof generation baseline",
            config=config,
            logger=logger,
        )
        self.llm = LLM(config, logger)
        self.immutable_funcs = immutable_funcs or []
        self.run_timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.input_name = None  # Will be set from environment or context

        # Naive baseline instruction - minimal guidance
        self.baseline_instruction = """You are helping to complete Verus code. The code contains TODO comments that need to be filled in.

Your task: Replace all TODO comments with the appropriate Verus specifications and proofs to make the code verify successfully.

This may include:
- Adding requires/ensures clauses to functions
- Implementing invariant functions
- Adding loop invariants
- Adding proof blocks with assertions

Return the complete, corrected Rust code. Do not include explanations."""

    def _get_llm_responses(
        self,
        instruction: str,
        code: str,
        examples: List[Dict[str, str]] = None,
        retry_attempt: int = 0,
        use_cache: bool = True,
    ) -> List[str]:
        """Get responses from LLM with error handling."""
        try:
            # Add retry marker to instruction to ensure cache miss for retries
            if retry_attempt > 0:
                instruction = (
                    f"{instruction}\n[Baseline Retry Attempt: {retry_attempt}]"
                )
                use_cache = False  # Disable cache for retries

            # Log the query details
            self.logger.info("=== Baseline LLM Query ===")
            self.logger.info(f"Retry Attempt: {retry_attempt}")
            self.logger.info(
                f"Model: {self.config.get('aoai_generation_model', 'gpt-4')}"
            )
            self.logger.info(f"Temperature: {0.7 + (retry_attempt * 0.1)}")
            self.logger.info(f"Answer Num: 5")
            self.logger.info(f"Max Tokens: {self.config.get('max_token', 16384)}")
            self.logger.info(f"Cache Enabled: {use_cache}")
            self.logger.info(f"Instruction Length: {len(instruction)} chars")
            self.logger.info(f"Code Length: {len(code)} chars")
            self.logger.info(f"Examples: {len(examples or [])}")
            self.logger.info("========================")

            # Check if model supports temperature (o1/o3 models don't)
            model_name = self.config.get("aoai_generation_model", "gpt-4")
            supports_temp = not any(x in model_name.lower() for x in ["o1", "o3"])
            temp_value = (0.7 + (retry_attempt * 0.1)) if supports_temp else None

            if not supports_temp:
                self.logger.info(
                    f"Model {model_name} doesn't support temperature, skipping temp parameter"
                )

            result = self.llm.infer_llm(
                model_name,
                instruction,
                examples or [],
                code,
                system_info="You are a helpful AI assistant specialized in Verus formal verification.",
                answer_num=5,  # Generate multiple candidates
                max_tokens=self.config.get("max_token", 16384),
                temp=temp_value,  # None for o1/o3 models
                use_cache=use_cache,
            )

            self.logger.info(f"LLM returned {len(result) if result else 0} responses")
            return result

        except Exception as e:
            self.logger.error(f"Error during baseline LLM inference: {e}")
            import traceback

            self.logger.error(f"Traceback: {traceback.format_exc()}")
            return []

    def _save_candidate_code(
        self,
        output_dir: Path,
        candidate_code: str,
        candidate_idx: int,
        attempt_num: int,
        metadata: Dict = None,
    ) -> Path:
        """
        Save a parsed candidate code to disk with metadata.

        Args:
            output_dir: Directory to save the file
            candidate_code: The parsed candidate code
            candidate_idx: Index of the candidate (1-based)
            attempt_num: Attempt number (1-based)
            metadata: Optional metadata dict to save alongside

        Returns:
            Path to the saved file
        """
        # Save the code with input name
        code_filename = f"baseline_{self.input_name}_candidate_{candidate_idx}_attempt_{attempt_num}.rs"
        code_path = output_dir / code_filename
        try:
            code_path.write_text(candidate_code)
            self.logger.info(
                f"Saved parsed candidate {candidate_idx} (attempt {attempt_num}) to {code_path}"
            )
        except Exception as e:
            self.logger.error(f"Error saving candidate code: {e}")

        # Save metadata if provided
        if metadata:
            meta_filename = f"baseline_{self.input_name}_candidate_{candidate_idx}_attempt_{attempt_num}_metadata.json"
            meta_path = output_dir / meta_filename
            try:
                with open(meta_path, "w") as f:
                    json.dump(metadata, f, indent=2, default=str)
                self.logger.debug(f"Saved metadata to {meta_path}")
            except Exception as e:
                self.logger.error(f"Error saving metadata: {e}")

        return code_path

    def _save_evaluation_result(
        self,
        output_dir: Path,
        candidate_idx: int,
        attempt_num: int,
        score,
        veval: VEval,
        is_best: bool = False,
    ):
        """
        Save evaluation results for a candidate.

        Args:
            output_dir: Directory to save the file
            candidate_idx: Index of the candidate (1-based)
            attempt_num: Attempt number (1-based)
            score: EvalScore object
            veval: VEval object with error details
            is_best: Whether this is currently the best candidate
        """
        eval_filename = f"baseline_{self.input_name}_eval_{candidate_idx}_attempt_{attempt_num}.json"
        eval_path = output_dir / eval_filename

        eval_data = {
            "candidate_idx": candidate_idx,
            "attempt_num": attempt_num,
            "timestamp": datetime.now().isoformat(),
            "score": {
                "verified": score.verified,
                "errors": score.errors,
                "verus_errors": score.verus_errors,
                "compilation_error": score.compilation_error,
                "is_correct": score.is_correct(),
            },
            "is_best_so_far": is_best,
        }

        # Add error details if available
        try:
            if hasattr(veval, "get_errs") and callable(veval.get_errs):
                errors = veval.get_errs()
                if errors:
                    eval_data["error_details"] = errors
        except Exception as e:
            self.logger.debug(f"Could not extract error details: {e}")

        try:
            with open(eval_path, "w") as f:
                json.dump(eval_data, f, indent=2, default=str)
            self.logger.debug(f"Saved evaluation results to {eval_path}")
        except Exception as e:
            self.logger.error(f"Error saving evaluation results: {e}")

    def _save_best_code(
        self,
        output_dir: Path,
        best_code: str,
        best_score,
        candidate_idx: int,
        attempt_num: int,
    ):
        """
        Save the current best code to a persistent location.

        Args:
            output_dir: Directory to save the file
            best_code: The best code so far
            best_score: Score of the best code
            candidate_idx: Index of the candidate that produced this
            attempt_num: Attempt number
        """
        # Save to main output directory (will persist) with input name
        best_path = output_dir / f"baseline_{self.input_name}_best_current.rs"
        best_with_metadata = (
            f"{best_code}\n\n"
            f"// BASELINE BEST CODE\n"
            f"// Source: Candidate {candidate_idx}, Attempt {attempt_num}\n"
            f"// Score: Verified={best_score.verified}, Errors={best_score.errors}, "
            f"Verus Errors={best_score.verus_errors}\n"
            f"// Compilation Error: {best_score.compilation_error}\n"
            f"// Is Correct: {best_score.is_correct()}\n"
            f"// Timestamp: {datetime.now().isoformat()}\n"
        )

        try:
            best_path.write_text(best_with_metadata)
            self.logger.info(f"Updated best code at {best_path}")
        except Exception as e:
            self.logger.error(f"Error saving best code: {e}")

    def _save_baseline_summary(
        self,
        output_dir: Path,
        best_code: str,
        best_score,
        total_attempts: int,
        total_candidates: int,
        successful_candidate: Dict = None,
        per_attempt_stats: List[Dict] = None,
    ):
        """
        Save a comprehensive summary of the baseline run.

        Args:
            output_dir: Directory to save the file (benchmark-specific subdirectory)
            best_code: The final best code
            best_score: Score of the best code
            total_attempts: Total number of attempts made
            total_candidates: Total number of candidates evaluated
            successful_candidate: Dict with details of successful candidate if any
            per_attempt_stats: List of per-attempt statistics for plotting
        """
        summary_path = output_dir / f"baseline_{self.input_name}_summary.json"

        summary = {
            "run_timestamp": self.run_timestamp,
            "completed_at": datetime.now().isoformat(),
            "total_attempts": total_attempts,
            "total_candidates_evaluated": total_candidates,
            "final_score": {
                "verified": best_score.verified if best_score else -1,
                "errors": best_score.errors if best_score else 999,
                "verus_errors": best_score.verus_errors if best_score else 999,
                "compilation_error": best_score.compilation_error
                if best_score
                else True,
                "is_correct": best_score.is_correct() if best_score else False,
            },
            "success": best_score.is_correct() if best_score else False,
        }

        # Add per-attempt statistics for plotting
        if per_attempt_stats:
            summary["per_attempt_stats"] = per_attempt_stats

            # Add aggregated timing info
            total_llm_time = sum(
                a.get("llm_time_seconds", 0) for a in per_attempt_stats
            )
            total_eval_time = (
                sum(a.get("total_time_seconds", 0) for a in per_attempt_stats)
                - total_llm_time
            )
            summary["timing"] = {
                "total_llm_time_seconds": total_llm_time,
                "total_eval_time_seconds": total_eval_time,
                "total_time_seconds": sum(
                    a.get("total_time_seconds", 0) for a in per_attempt_stats
                ),
                "average_llm_time_per_attempt": total_llm_time / len(per_attempt_stats)
                if per_attempt_stats
                else 0,
            }

        if successful_candidate:
            summary["successful_candidate"] = successful_candidate

        try:
            with open(summary_path, "w") as f:
                json.dump(summary, f, indent=2, default=str)
            self.logger.info(f"Saved baseline summary to {summary_path}")

            # Also save the final best code
            final_code_path = output_dir / f"baseline_{self.input_name}_final_result.rs"
            final_code_with_metadata = (
                f"{best_code}\n\n"
                f"// BASELINE FINAL RESULT - {self.input_name}\n"
                f"// Total Attempts: {total_attempts}\n"
                f"// Total Candidates: {total_candidates}\n"
                f"// Final Score: {best_score}\n"
                f"// Success: {best_score.is_correct() if best_score else False}\n"
                f"// Completed: {datetime.now().isoformat()}\n"
            )
            final_code_path.write_text(final_code_with_metadata)
            self.logger.info(f"Saved final baseline result to {final_code_path}")

        except Exception as e:
            self.logger.error(f"Error saving baseline summary: {e}")

    def exec(self, context) -> str:
        """
        Execute the baseline module with a single comprehensive LLM call.

        Args:
            context: Context object containing trial information

        Returns:
            Generated code with specifications and proofs
        """
        self.logger.info("=== Baseline Single-Shot Generation ===")

        # Get the initial todo code
        code = context.trials[-1].code
        original_code = code

        # Extract input file name for file naming
        import os
        from pathlib import Path

        input_file = os.environ.get("VERUS_TEST_FILE", "unknown")
        self.input_name = Path(input_file).stem  # e.g., "atomics_todo"
        self.logger.info(f"Input file name for baseline: {self.input_name}")

        # Allow configuring max retries via environment variable
        max_retries = int(os.environ.get("VERUS_BASELINE_MAX_RETRIES", "10"))
        self.logger.info(f"Max retries configured: {max_retries}")
        best_code = code
        best_score = None
        best_veval = None
        total_candidates_evaluated = 0
        successful_candidate_info = None

        # Track per-attempt statistics for plotting
        per_attempt_stats = []

        # Create benchmark-specific subdirectory early (before attempts)
        base_samples_dir = samples_dir()
        output_dir = base_samples_dir / self.input_name
        output_dir.mkdir(exist_ok=True, parents=True)
        self.logger.info(f"Output directory for baseline: {output_dir}")

        for retry_attempt in range(max_retries):
            self.logger.info(f"Baseline attempt {retry_attempt + 1}/{max_retries}")

            # Track attempt-level statistics
            attempt_start_time = datetime.now()
            attempt_candidates = []

            # Get examples if available (but don't require them for baseline)
            examples = []
            try:
                examples = get_examples(self.config, "baseline", self.logger)
            except Exception as e:
                self.logger.debug(f"No baseline examples found: {e}")

            # Build instruction with error feedback from previous attempt
            instruction = self.baseline_instruction

            # Add error feedback for retry attempts
            if retry_attempt > 0 and best_veval is not None:
                try:
                    error_info = best_veval.get_error_info()
                    if error_info:
                        feedback = f"\n\nPREVIOUS ATTEMPT ERRORS:\n{error_info}\n\nPlease fix these errors in your solution."
                        instruction += feedback
                        self.logger.info(
                            f"Added error feedback from previous attempt ({len(error_info)} chars)"
                        )
                except Exception as e:
                    self.logger.debug(f"Could not extract error feedback: {e}")

            self.logger.info("Using naive baseline instruction (no syntax guidance)")

            # Save prompt for debugging
            prompt_file = prompt_dir() / f"baseline_{retry_attempt + 1}.txt"
            prompt_file.write_text(instruction + "\n\n---\n\n" + code)
            self.logger.info(f"Saved baseline prompt to {prompt_file}")

            # Get LLM responses
            llm_start_time = datetime.now()
            responses = self._get_llm_responses(
                instruction,
                code,
                examples=examples,
                retry_attempt=retry_attempt,
                use_cache=(retry_attempt == 0),
            )
            llm_end_time = datetime.now()
            llm_time = (llm_end_time - llm_start_time).total_seconds()

            if not responses:
                self.logger.warning(
                    f"No responses from LLM on attempt {retry_attempt + 1}"
                )
                # Save attempt stats even if failed
                per_attempt_stats.append(
                    {
                        "attempt": retry_attempt + 1,
                        "llm_time": llm_time,
                        "total_time": (
                            datetime.now() - attempt_start_time
                        ).total_seconds(),
                        "candidates": [],
                        "best_verus_errors": None,
                        "success": False,
                    }
                )
                continue

            # Process each response and find the best one
            self.logger.info(f"Processing {len(responses)} responses from LLM")
            for i, response in enumerate(responses):
                try:
                    candidate_num = i + 1
                    attempt_num = retry_attempt + 1

                    # Save raw sample
                    sample_path = (
                        output_dir
                        / f"baseline_raw_sample_{candidate_num}_attempt_{attempt_num}.rs"
                    )
                    sample_path.write_text(response)
                    self.logger.info(
                        f"Saved baseline raw sample {candidate_num} from attempt {attempt_num} to {sample_path}"
                    )

                    # Parse the response to extract code
                    candidate_code = parse_llm_response(response, self.logger)
                    if not candidate_code.strip():
                        self.logger.warning(
                            f"Empty candidate code from response {candidate_num}"
                        )
                        continue

                    # Save parsed candidate code with metadata
                    self._save_candidate_code(
                        output_dir,
                        candidate_code,
                        candidate_num,
                        attempt_num,
                        metadata={
                            "raw_response_length": len(response),
                            "parsed_code_length": len(candidate_code),
                            "timestamp": datetime.now().isoformat(),
                        },
                    )

                    # Check safety if we have immutable functions
                    if self.immutable_funcs and not code_change_is_safe(
                        original_code,
                        candidate_code,
                        verus_path=self.config.get("verus_path", "verus"),
                        logger=self.logger,
                        immutable_funcs=self.immutable_funcs,
                    ):
                        self.logger.warning(
                            f"Unsafe code change detected in candidate {candidate_num}, skipping"
                        )
                        continue

                    # Evaluate the candidate
                    self.logger.info(
                        f"Evaluating baseline candidate {candidate_num} from attempt {attempt_num}"
                    )
                    veval = VEval(candidate_code, self.logger)
                    score = veval.eval_and_get_score()
                    total_candidates_evaluated += 1

                    self.logger.info(f"Candidate {candidate_num} score: {score}")

                    # Track candidate stats for this attempt
                    attempt_candidates.append(
                        {
                            "candidate_num": candidate_num,
                            "verified": score.verified,
                            "errors": score.errors,
                            "verus_errors": score.verus_errors,
                            "compilation_error": score.compilation_error,
                            "is_correct": score.is_correct(),
                        }
                    )

                    # Check if this is the best so far
                    is_new_best = best_score is None or score > best_score

                    # Save evaluation results
                    self._save_evaluation_result(
                        output_dir,
                        candidate_num,
                        attempt_num,
                        score,
                        veval,
                        is_best=is_new_best,
                    )

                    if is_new_best:
                        best_score = score
                        best_code = candidate_code
                        self.logger.info(
                            f"New best baseline candidate with score: {score}"
                        )

                        # Save the new best code
                        self._save_best_code(
                            output_dir,
                            best_code,
                            best_score,
                            candidate_num,
                            attempt_num,
                        )

                        # Add trial to context
                        from src.context import Trial

                        trial_id = len(context.trials)
                        tmp_dir = self.config.get("tmp_dir", "tmp")
                        trial_path = os.path.join(
                            tmp_dir, f"baseline_trial_{trial_id}.rs"
                        )
                        with open(trial_path, "w") as f:
                            f.write(candidate_code)
                        trial = Trial(trial_id, veval, trial_path, self.logger)
                        context.trials.append(trial)

                        # Update checkpoint best
                        context.best_code = best_code
                        context.best_score = best_score

                        # If we found a correct solution, save summary and return
                        if score.is_correct():
                            self.logger.info("🎉 Found correct baseline solution!")
                            successful_candidate_info = {
                                "candidate_idx": candidate_num,
                                "attempt_num": attempt_num,
                                "score": {
                                    "verified": score.verified,
                                    "errors": score.errors,
                                    "verus_errors": score.verus_errors,
                                    "compilation_error": score.compilation_error,
                                },
                            }
                            self._save_baseline_summary(
                                output_dir,
                                best_code,
                                best_score,
                                retry_attempt + 1,
                                total_candidates_evaluated,
                                successful_candidate_info,
                                per_attempt_stats,
                            )
                            return candidate_code

                except Exception as e:
                    self.logger.error(
                        f"Error evaluating candidate {candidate_num}: {e}"
                    )
                    import traceback

                    self.logger.debug(f"Traceback: {traceback.format_exc()}")
                    continue

            # Save per-attempt statistics
            attempt_end_time = datetime.now()
            attempt_total_time = (attempt_end_time - attempt_start_time).total_seconds()

            # Find best candidate in this attempt
            attempt_best_verus_errors = None
            attempt_best_candidate_num = None
            if attempt_candidates:
                attempt_best_verus_errors = min(
                    [c["verus_errors"] for c in attempt_candidates]
                )
                for c in attempt_candidates:
                    if c["verus_errors"] == attempt_best_verus_errors:
                        attempt_best_candidate_num = c["candidate_num"]
                        break

            per_attempt_stats.append(
                {
                    "attempt": retry_attempt + 1,
                    "llm_time_seconds": llm_time,
                    "total_time_seconds": attempt_total_time,
                    "eval_time_seconds": attempt_total_time - llm_time,
                    "num_candidates": len(attempt_candidates),
                    "candidates": attempt_candidates,
                    "best_verus_errors": attempt_best_verus_errors,
                    "best_candidate_num": attempt_best_candidate_num,
                    "timestamp": attempt_start_time.isoformat(),
                }
            )

            # If we have a good score, we can try to return early
            if best_score and best_score.verified > 0:
                self.logger.info(
                    f"Found good baseline code with score {best_score}, stopping early"
                )
                break

        # Log final result and save summary
        if best_score:
            self.logger.info(f"Baseline completed with best score: {best_score}")
        else:
            self.logger.warning("Baseline failed to generate any valid candidates")
            best_code = code  # Return original if nothing worked

        # Save final summary
        self._save_baseline_summary(
            output_dir,
            best_code,
            best_score,
            max_retries,
            total_candidates_evaluated,
            successful_candidate_info,
            per_attempt_stats,
        )

        return best_code
