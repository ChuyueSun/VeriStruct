import json
import logging
import os
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from src.modules.statistics_collector import StatisticsCollector
from src.modules.veval import EvalScore


class ProgressLogger:
    """
    Tracks and logs the progress of VeriStruct execution, including:
    - Step timing
    - VEval results after each step
    - Repair information for each round
    """

    def __init__(self, output_dir: Path, logger: logging.Logger):
        """
        Initialize the progress logger.

        Args:
            output_dir: Directory where logs will be saved
            logger: Logger instance for standard logging
        """
        self.output_dir = output_dir
        self.logger = logger
        self.progress = {
            "start_time": datetime.now().isoformat(),
            "steps": [],
            "repair_rounds": [],
            "final_result": None,
            "total_execution_time": None,
        }
        self.current_step = None
        self.current_step_start_time = None

        # Create log directory
        self.log_dir = output_dir / "progress_logs"
        self.log_dir.mkdir(exist_ok=True, parents=True)

        # Create timestamp for unique filenames
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # Get input file name from environment if available
        input_file = os.environ.get("VERUS_INPUT_FILE", "")
        if input_file:
            self.file_id = f"{input_file}_{self.timestamp}"
        else:
            self.file_id = self.timestamp

        # Log file paths with file ID
        self.log_file = self.log_dir / f"progress_{self.file_id}.json"

        self.logger.info(f"Progress logger initialized. Logs will be saved to {self.log_file}")

        # Initialize statistics collector
        benchmark_name = os.environ.get("VERUS_INPUT_FILE", "unknown")
        self.stats_collector = StatisticsCollector(output_dir, benchmark_name, logger)
        self.logger.info("Statistics collector initialized")

        # Display file identification summary
        self.display_file_info()

    def display_file_info(self) -> None:
        """Display a summary of input and output file information for reference."""
        input_file = os.environ.get("VERUS_INPUT_FILE", "Unknown")
        file_id = os.environ.get("VERUS_FILE_ID", self.file_id)

        self.logger.info("=" * 50)
        self.logger.info("FILE IDENTIFICATION SUMMARY")
        self.logger.info("-" * 50)
        self.logger.info(f"Input File: {input_file}")
        self.logger.info(f"File ID for outputs: {file_id}")
        self.logger.info(f"Timestamp: {self.timestamp}")
        self.logger.info(f"Progress Log: {self.log_file}")
        self.logger.info(f"Output Directory: {self.output_dir.absolute()}")
        self.logger.info("=" * 50)

    def start_step(self, step_name: str, step_number: int) -> None:
        """
        Mark the start of a processing step.

        Args:
            step_name: Name of the step (e.g., "view_inference")
            step_number: Number of the step in the sequence
        """
        self.current_step = {
            "name": step_name,
            "number": step_number,
            "start_time": datetime.now().isoformat(),
            "result": None,
            "execution_time": None,
        }
        self.current_step_start_time = time.time()
        self.logger.info(f"Starting step {step_number}: {step_name}")

        # Track in statistics collector
        self.stats_collector.start_stage(step_name, step_number)

        self._save_progress()

    def end_step(self, result_score: EvalScore, result_length: int) -> None:
        """
        Mark the end of a processing step and record results.

        Args:
            result_score: EvalScore from the step
            result_length: Length of the generated code
        """
        if self.current_step is None:
            self.logger.warning("Attempting to end a step, but no step is in progress")
            return

        end_time = time.time()
        execution_time = end_time - self.current_step_start_time

        self.current_step["result"] = {
            "compilation_error": result_score.compilation_error,
            "verified": result_score.verified,
            "errors": result_score.errors,
            "verus_errors": result_score.verus_errors,
            "is_correct": result_score.is_correct(),
            "code_length": result_length,
        }
        self.current_step["execution_time"] = execution_time
        self.current_step["end_time"] = datetime.now().isoformat()

        self.progress["steps"].append(self.current_step)

        self.logger.info(
            f"Completed step {self.current_step['number']}: {self.current_step['name']} "
            f"in {execution_time:.2f}s with score: {result_score}"
        )

        # Track in statistics collector
        self.stats_collector.end_stage(self.current_step["name"], result_score)

        self.current_step = None
        self.current_step_start_time = None
        self._save_progress()

    def start_repair_round(self, round_number: int) -> None:
        """
        Start a new repair round.

        Args:
            round_number: The current repair round number
        """
        repair_round = {
            "round_number": round_number,
            "start_time": datetime.now().isoformat(),
            "repairs": [],
            "end_time": None,
            "execution_time": None,
        }

        self.progress["repair_rounds"].append(repair_round)
        self.logger.info(f"Starting repair round {round_number}")

        # Track in statistics collector
        self.stats_collector.start_repair_round(round_number)

        self._save_progress()

    def add_repair(
        self,
        error_type: str,
        repair_module: str,
        before_score: EvalScore,
        after_score: EvalScore,
        execution_time: float,
    ) -> None:
        """
        Add information about a repair that was performed.

        Args:
            error_type: Type of error that was repaired
            repair_module: Name of the repair module used
            before_score: Score before the repair
            after_score: Score after the repair
            execution_time: Time taken for the repair
        """
        if not self.progress["repair_rounds"]:
            self.logger.warning("Attempting to add a repair, but no repair round is in progress")
            return

        repair_round = self.progress["repair_rounds"][-1]

        repair = {
            "error_type": error_type,
            "repair_module": repair_module,
            "before_score": {
                "compilation_error": before_score.compilation_error,
                "verified": before_score.verified,
                "errors": before_score.errors,
                "verus_errors": before_score.verus_errors,
            },
            "after_score": {
                "compilation_error": after_score.compilation_error,
                "verified": after_score.verified,
                "errors": after_score.errors,
                "verus_errors": after_score.verus_errors,
            },
            "success": after_score > before_score,
            "execution_time": execution_time,
        }

        repair_round["repairs"].append(repair)

        self.logger.info(
            f"Completed repair {repair_module} for {error_type} "
            f"in {execution_time:.2f}s. Score improved from {before_score} to {after_score}"
        )

        # Track in statistics collector
        success = after_score > before_score
        self.stats_collector.record_repair(
            error_type,
            repair_module,
            execution_time,
            success,
            before_score,
            after_score,
        )

        self._save_progress()

    def end_repair_round(self) -> None:
        """End the current repair round and record timing information."""
        if not self.progress["repair_rounds"]:
            self.logger.warning("Attempting to end a repair round, but no round is in progress")
            return

        repair_round = self.progress["repair_rounds"][-1]

        if repair_round.get("end_time") is not None:
            self.logger.warning(f"Repair round {repair_round['round_number']} already ended")
            return

        start_time = datetime.fromisoformat(repair_round["start_time"])
        end_time = datetime.now()
        execution_time = (end_time - start_time).total_seconds()

        repair_round["end_time"] = end_time.isoformat()
        repair_round["execution_time"] = execution_time

        repairs_used = [r["repair_module"] for r in repair_round["repairs"]]
        errors_fixed = [r["error_type"] for r in repair_round["repairs"] if r["success"]]

        self.logger.info(
            f"Completed repair round {repair_round['round_number']} in {execution_time:.2f}s. "
            f"Repairs used: {', '.join(repairs_used)}. Errors fixed: {', '.join(errors_fixed)}"
        )
        self._save_progress()

    def record_final_result(self, final_score: EvalScore, final_code: str = None) -> None:
        """
        Record the final verification result.

        Args:
            final_score: The final EvalScore after all processing
            final_code: The final code (optional)
        """
        self.progress["final_result"] = {
            "compilation_error": final_score.compilation_error,
            "verified": final_score.verified,
            "errors": final_score.errors,
            "verus_errors": final_score.verus_errors,
            "is_correct": final_score.is_correct(),
        }

        start_time = datetime.fromisoformat(self.progress["start_time"])
        end_time = datetime.now()
        total_time = (end_time - start_time).total_seconds()

        self.progress["end_time"] = end_time.isoformat()
        self.progress["total_execution_time"] = total_time

        self.logger.info(
            f"VeriStruct completed in {total_time:.2f}s with final score: {final_score}"
        )

        # Record final state in statistics collector
        if final_code:
            self.stats_collector.record_final_state(final_code, final_score)

        self._save_progress()

        # Also create a summary file with key metrics
        self._save_summary()

        # Save detailed statistics
        self._save_statistics()

    def _save_progress(self) -> None:
        """Save the current progress to the JSON log file."""
        try:
            with open(self.log_file, "w") as f:
                json.dump(self.progress, f, indent=2)
        except Exception as e:
            self.logger.error(f"Error saving progress log: {e}")

    def _save_summary(self) -> None:
        """Save a summary of the execution to a text file."""
        try:
            summary_file = self.log_dir / f"summary_{self.file_id}.txt"

            # Calculate some statistics
            total_steps = len(self.progress["steps"])
            total_repair_rounds = len(self.progress["repair_rounds"])
            total_repairs = sum(len(round["repairs"]) for round in self.progress["repair_rounds"])
            successful_repairs = sum(
                sum(1 for repair in round["repairs"] if repair["success"])
                for round in self.progress["repair_rounds"]
            )

            step_times = [
                step["execution_time"]
                for step in self.progress["steps"]
                if step["execution_time"] is not None
            ]
            avg_step_time = sum(step_times) / len(step_times) if step_times else 0

            repair_times = [
                repair["execution_time"]
                for round in self.progress["repair_rounds"]
                for repair in round["repairs"]
                if "execution_time" in repair
            ]
            avg_repair_time = sum(repair_times) / len(repair_times) if repair_times else 0

            # Get input file info
            input_file = os.environ.get("VERUS_TEST_FILE", "Unknown")
            input_file_name = os.path.basename(input_file) if input_file != "Unknown" else "Unknown"
            file_id = os.environ.get("VERUS_FILE_ID", self.file_id)

            # Write summary
            with open(summary_file, "w") as f:
                f.write("# VeriStruct Execution Summary\n\n")

                # Add input file information
                f.write("## Input and Output Files\n\n")
                f.write(f"Input File: {input_file}\n")
                f.write(f"Input File Name: {input_file_name}\n")
                f.write(f"File ID: {file_id}\n")
                f.write(f"Output Directory: {self.output_dir.absolute()}\n")
                f.write(f"Progress Log: {self.log_file}\n")
                f.write(f"Summary File: {summary_file}\n\n")

                f.write(f"Start time: {self.progress['start_time']}\n")
                if "end_time" in self.progress:
                    f.write(f"End time: {self.progress['end_time']}\n")
                if "total_execution_time" in self.progress:
                    f.write(
                        f"Total execution time: {self.progress['total_execution_time']:.2f}s\n\n"
                    )

                f.write("## Final Result\n\n")
                if self.progress["final_result"]:
                    fr = self.progress["final_result"]
                    f.write(f"Verified: {fr['verified']}\n")
                    f.write(f"Errors: {fr['errors']}\n")
                    f.write(f"Verus Errors: {fr['verus_errors']}\n")
                    f.write(f"Compilation Error: {fr['compilation_error']}\n")
                    f.write(f"Is Correct: {fr['is_correct']}\n\n")

                f.write("## Statistics\n\n")
                f.write(f"Total steps: {total_steps}\n")
                f.write(f"Total repair rounds: {total_repair_rounds}\n")
                f.write(f"Total repairs attempted: {total_repairs}\n")
                f.write(f"Successful repairs: {successful_repairs}\n")
                f.write(f"Average step time: {avg_step_time:.2f}s\n")
                f.write(f"Average repair time: {avg_repair_time:.2f}s\n\n")

                f.write("## Steps\n\n")
                for step in self.progress["steps"]:
                    f.write(f"Step {step['number']}: {step['name']}\n")
                    if "execution_time" in step and step["execution_time"] is not None:
                        f.write(f"  Time: {step['execution_time']:.2f}s\n")
                    if "result" in step and step["result"] is not None:
                        r = step["result"]
                        f.write(
                            f"  Score: Verified={r['verified']}, Errors={r['errors']}, Verus Errors={r['verus_errors']}\n"
                        )
                f.write("\n")

                f.write("## Repair Rounds\n\n")
                for round in self.progress["repair_rounds"]:
                    f.write(f"Round {round['round_number']}\n")
                    if "execution_time" in round and round["execution_time"] is not None:
                        f.write(f"  Time: {round['execution_time']:.2f}s\n")

                    for repair in round["repairs"]:
                        before = repair["before_score"]
                        after = repair["after_score"]
                        f.write(f"  {repair['repair_module']} for {repair['error_type']}\n")
                        f.write(
                            f"    Before: Verified={before['verified']}, Errors={before['errors']}, Verus Errors={before['verus_errors']}\n"
                        )
                        f.write(
                            f"    After: Verified={after['verified']}, Errors={after['errors']}, Verus Errors={after['verus_errors']}\n"
                        )
                        if "execution_time" in repair:
                            f.write(f"    Time: {repair['execution_time']:.2f}s\n")
                    f.write("\n")

        except Exception as e:
            self.logger.error(f"Error saving summary: {e}")

    def _save_statistics(self) -> None:
        """Save detailed statistics collected during execution."""
        try:
            detailed_file, summary_file, report_file = self.stats_collector.save()
            self.logger.info(f"Statistics saved:")
            self.logger.info(f"  - Detailed: {detailed_file}")
            self.logger.info(f"  - Summary: {summary_file}")
            self.logger.info(f"  - Report: {report_file}")
        except Exception as e:
            self.logger.error(f"Error saving statistics: {e}")

    def record_initial_state(self, code: str, eval_score: EvalScore, failures: List = None):
        """
        Record the initial state of the benchmark.

        Args:
            code: Initial code
            eval_score: Initial evaluation score
            failures: List of initial failures
        """
        self.stats_collector.record_initial_state(code, eval_score, failures)

    def record_llm_call(
        self,
        stage: Optional[str] = None,
        module: Optional[str] = None,
        response_time: Optional[float] = None,
        cache_hit: bool = False,
        input_tokens: Optional[int] = None,
        output_tokens: Optional[int] = None,
    ):
        """
        Record an LLM call for statistics.

        Args:
            stage: Stage/module name where the call was made
            module: Specific module name (if different from stage)
            response_time: Time taken for the LLM call
            cache_hit: Whether this was a cache hit
        """
        self.stats_collector.record_llm_call(
            stage,
            module,
            response_time,
            cache_hit,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
        )
