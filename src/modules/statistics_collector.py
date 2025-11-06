"""
Enhanced Statistics Collection System for VerusAgent

This module tracks detailed statistics for research paper reporting:
- Number of LLM calls per stage/module
- Number of iterations in each module
- Number of modules activated
- Types of repair heuristics triggered
- Response times for each operation
- Success rates and error distributions
"""

import json
import time
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Set

from src.modules.veval import EvalScore, VerusErrorType


class StatisticsCollector:
    """
    Collects detailed statistics during VerusAgent execution for research analysis.
    """

    def __init__(self, output_dir: Path, benchmark_name: str, logger):
        """
        Initialize the statistics collector.

        Args:
            output_dir: Directory where statistics will be saved
            benchmark_name: Name of the benchmark being processed
            logger: Logger instance
        """
        self.output_dir = output_dir
        self.benchmark_name = benchmark_name
        self.logger = logger
        self.start_time = time.time()

        # Statistics data structure
        self.stats = {
            "benchmark_name": benchmark_name,
            "start_time": datetime.now().isoformat(),
            "end_time": None,
            "total_execution_time": None,
            # Module activation tracking
            "modules_activated": [],
            "modules_count": 0,
            # Stage-wise statistics
            "stages": {},  # stage_name -> stage_stats
            # LLM call tracking
            "llm_calls": {
                "total": 0,
                "by_stage": defaultdict(int),
                "by_module": defaultdict(int),
                "response_times": [],  # List of (stage, module, time) tuples
                "cache_hits": 0,
                "cache_misses": 0,
                # Token usage tracking
                "input_tokens": 0,
                "output_tokens": 0,
                "by_call": [],  # List of per-call token usage with context
            },
            # Iteration tracking
            "iterations": {
                "by_module": defaultdict(int),
                "by_stage": defaultdict(int),
            },
            # Repair tracking
            "repairs": {
                "total_rounds": 0,
                "total_repairs": 0,
                "repairs_by_type": defaultdict(int),
                "repairs_by_heuristic": defaultdict(int),
                "successful_repairs": 0,
                "failed_repairs": 0,
                "repair_times": [],  # List of (repair_type, heuristic, time, success) tuples
            },
            # Error tracking
            "errors": {
                "initial_error_count": 0,
                "final_error_count": 0,
                "errors_by_type": defaultdict(int),
                "errors_fixed_by_type": defaultdict(int),
            },
            # Verification results
            "verification": {
                "initial_verified": 0,
                "final_verified": 0,
                "initial_errors": 0,
                "final_errors": 0,
                "compilation_errors": 0,
            },
            # Code metrics
            "code_metrics": {
                "initial_code_length": 0,
                "final_code_length": 0,
                "code_changes": 0,
            },
        }

        # Create statistics directory
        self.stats_dir = output_dir / "statistics"
        self.stats_dir.mkdir(exist_ok=True, parents=True)

        # Tracking state
        self.current_stage = None
        self.current_module = None
        self.stage_start_time = None
        self.module_start_time = None

    def start_stage(self, stage_name: str, step_number: int):
        """
        Mark the start of a processing stage.

        Args:
            stage_name: Name of the stage
            step_number: Step number in the execution
        """
        self.current_stage = stage_name
        self.stage_start_time = time.time()

        # Add to modules activated if not already present
        if stage_name not in self.stats["modules_activated"]:
            self.stats["modules_activated"].append(stage_name)
            self.stats["modules_count"] += 1

        # Initialize stage stats
        if stage_name not in self.stats["stages"]:
            self.stats["stages"][stage_name] = {
                "step_number": step_number,
                "start_time": datetime.now().isoformat(),
                "end_time": None,
                "execution_time": None,
                "llm_calls": 0,
                "iterations": 0,
                "result": None,
            }

        self.logger.debug(f"Statistics: Started tracking stage {stage_name}")

    def end_stage(
        self,
        stage_name: str,
        result_score: Optional[EvalScore] = None,
        iterations: int = 1,
    ):
        """
        Mark the end of a processing stage.

        Args:
            stage_name: Name of the stage
            result_score: Optional evaluation score after the stage
            iterations: Number of iterations performed in this stage
        """
        if stage_name not in self.stats["stages"]:
            self.logger.warning(f"Attempting to end stage {stage_name} that was not started")
            return

        stage = self.stats["stages"][stage_name]
        stage["end_time"] = datetime.now().isoformat()

        if self.stage_start_time:
            stage["execution_time"] = time.time() - self.stage_start_time

        stage["iterations"] = iterations
        self.stats["iterations"]["by_stage"][stage_name] += iterations

        if result_score:
            stage["result"] = {
                "compilation_error": result_score.compilation_error,
                "verified": result_score.verified,
                "errors": result_score.errors,
                "verus_errors": result_score.verus_errors,
                "is_correct": result_score.is_correct(),
            }

        self.current_stage = None
        self.stage_start_time = None
        self.logger.debug(f"Statistics: Ended tracking stage {stage_name}")

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
        Record an LLM call.

        Args:
            stage: Stage/module name where the call was made
            module: Specific module name (if different from stage)
            response_time: Time taken for the LLM call
            cache_hit: Whether this was a cache hit
        """
        self.stats["llm_calls"]["total"] += 1

        stage_name = stage or self.current_stage or "unknown"
        module_name = module or stage_name

        self.stats["llm_calls"]["by_stage"][stage_name] += 1
        self.stats["llm_calls"]["by_module"][module_name] += 1

        if cache_hit:
            self.stats["llm_calls"]["cache_hits"] += 1
        else:
            self.stats["llm_calls"]["cache_misses"] += 1

        if response_time is not None:
            self.stats["llm_calls"]["response_times"].append(
                {
                    "stage": stage_name,
                    "module": module_name,
                    "time": response_time,
                    "cache_hit": cache_hit,
                }
            )

        # Track token usage if available
        if isinstance(input_tokens, int):
            self.stats["llm_calls"]["input_tokens"] += max(0, input_tokens)
        if isinstance(output_tokens, int):
            self.stats["llm_calls"]["output_tokens"] += max(0, output_tokens)
        if (isinstance(input_tokens, int)) or (isinstance(output_tokens, int)):
            self.stats["llm_calls"]["by_call"].append(
                {
                    "stage": stage_name,
                    "module": module_name,
                    "input_tokens": input_tokens,
                    "output_tokens": output_tokens,
                    "cache_hit": cache_hit,
                    "time": response_time,
                }
            )

        # Update stage-specific LLM call count
        if stage_name in self.stats["stages"]:
            self.stats["stages"][stage_name]["llm_calls"] += 1

    def record_iteration(self, module: str):
        """
        Record an iteration in a module.

        Args:
            module: Name of the module
        """
        self.stats["iterations"]["by_module"][module] += 1

    def start_repair_round(self, round_number: int):
        """
        Start a repair round.

        Args:
            round_number: The repair round number
        """
        self.stats["repairs"]["total_rounds"] = round_number
        self.logger.debug(f"Statistics: Started repair round {round_number}")

    def record_repair(
        self,
        error_type: str,
        repair_heuristic: str,
        repair_time: float,
        success: bool,
        before_score: Optional[EvalScore] = None,
        after_score: Optional[EvalScore] = None,
    ):
        """
        Record a repair attempt.

        Args:
            error_type: Type of error being repaired
            repair_heuristic: Name of the repair heuristic/module used
            repair_time: Time taken for the repair
            success: Whether the repair was successful
            before_score: Score before repair
            after_score: Score after repair
        """
        self.stats["repairs"]["total_repairs"] += 1
        self.stats["repairs"]["repairs_by_type"][error_type] += 1
        self.stats["repairs"]["repairs_by_heuristic"][repair_heuristic] += 1

        if success:
            self.stats["repairs"]["successful_repairs"] += 1
            self.stats["errors"]["errors_fixed_by_type"][error_type] += 1
        else:
            self.stats["repairs"]["failed_repairs"] += 1

        self.stats["repairs"]["repair_times"].append(
            {
                "error_type": error_type,
                "heuristic": repair_heuristic,
                "time": repair_time,
                "success": success,
                "before": {
                    "verified": before_score.verified if before_score else 0,
                    "errors": before_score.errors if before_score else 0,
                }
                if before_score
                else None,
                "after": {
                    "verified": after_score.verified if after_score else 0,
                    "errors": after_score.errors if after_score else 0,
                }
                if after_score
                else None,
            }
        )

    def record_initial_state(self, code: str, eval_score: EvalScore, failures: List = None):
        """
        Record the initial state of the benchmark.

        Args:
            code: Initial code
            eval_score: Initial evaluation score
            failures: List of initial failures
        """
        self.stats["code_metrics"]["initial_code_length"] = len(code)
        self.stats["verification"]["initial_verified"] = eval_score.verified
        self.stats["verification"]["initial_errors"] = eval_score.errors
        self.stats["errors"]["initial_error_count"] = eval_score.errors

        if failures:
            for failure in failures:
                error_type = (
                    failure.error.name if hasattr(failure.error, "name") else str(failure.error)
                )
                self.stats["errors"]["errors_by_type"][error_type] += 1

    def record_final_state(self, code: str, eval_score: EvalScore, failures: List = None):
        """
        Record the final state of the benchmark.

        Args:
            code: Final code
            eval_score: Final evaluation score
            failures: List of remaining failures
        """
        self.stats["code_metrics"]["final_code_length"] = len(code)
        self.stats["verification"]["final_verified"] = eval_score.verified
        self.stats["verification"]["final_errors"] = eval_score.errors
        self.stats["errors"]["final_error_count"] = eval_score.errors

        if eval_score.compilation_error:
            self.stats["verification"]["compilation_errors"] = 1

        # Calculate code changes
        initial_length = self.stats["code_metrics"]["initial_code_length"]
        final_length = self.stats["code_metrics"]["final_code_length"]
        self.stats["code_metrics"]["code_changes"] = abs(final_length - initial_length)

        # Record final time
        self.stats["end_time"] = datetime.now().isoformat()
        self.stats["total_execution_time"] = time.time() - self.start_time

    def get_summary(self) -> Dict[str, Any]:
        """
        Get a summary of the collected statistics.

        Returns:
            Dictionary containing summary statistics
        """
        # Calculate average response times
        response_times = [rt["time"] for rt in self.stats["llm_calls"]["response_times"]]
        avg_response_time = sum(response_times) / len(response_times) if response_times else 0

        # Calculate repair success rate
        total_repairs = self.stats["repairs"]["total_repairs"]
        successful_repairs = self.stats["repairs"]["successful_repairs"]
        repair_success_rate = (successful_repairs / total_repairs * 100) if total_repairs > 0 else 0

        # Calculate cache hit rate
        total_llm_calls = self.stats["llm_calls"]["total"]
        cache_hits = self.stats["llm_calls"]["cache_hits"]
        cache_hit_rate = (cache_hits / total_llm_calls * 100) if total_llm_calls > 0 else 0

        return {
            "benchmark": self.benchmark_name,
            "execution_time": self.stats.get("total_execution_time", 0),
            "modules_activated": self.stats["modules_count"],
            "total_llm_calls": total_llm_calls,
            "avg_response_time": avg_response_time,
            "cache_hit_rate": cache_hit_rate,
            "total_repair_rounds": self.stats["repairs"]["total_rounds"],
            "total_repairs": total_repairs,
            "repair_success_rate": repair_success_rate,
            "initial_errors": self.stats["errors"]["initial_error_count"],
            "final_errors": self.stats["errors"]["final_error_count"],
            "errors_fixed": self.stats["errors"]["initial_error_count"]
            - self.stats["errors"]["final_error_count"],
            "verification_success": self.stats["verification"]["final_errors"] == 0,
        }

    def save(self):
        """
        Save the collected statistics to files.
        """
        # Save detailed statistics as JSON
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        detailed_file = self.stats_dir / f"detailed_{self.benchmark_name}_{timestamp}.json"

        # Convert defaultdicts to regular dicts for JSON serialization
        stats_to_save = json.loads(
            json.dumps(
                self.stats,
                default=lambda x: dict(x) if isinstance(x, defaultdict) else x,
            )
        )

        with open(detailed_file, "w") as f:
            json.dump(stats_to_save, f, indent=2)

        self.logger.info(f"Saved detailed statistics to {detailed_file}")

        # Save summary statistics
        summary = self.get_summary()
        summary_file = self.stats_dir / f"summary_{self.benchmark_name}_{timestamp}.json"
        with open(summary_file, "w") as f:
            json.dump(summary, f, indent=2)

        self.logger.info(f"Saved summary statistics to {summary_file}")

        # Save human-readable report
        report_file = self.stats_dir / f"report_{self.benchmark_name}_{timestamp}.txt"
        self._save_human_readable_report(report_file, summary)

        return detailed_file, summary_file, report_file

    def _save_human_readable_report(self, report_file: Path, summary: Dict[str, Any]):
        """
        Save a human-readable statistics report.

        Args:
            report_file: Path to save the report
            summary: Summary statistics dictionary
        """
        with open(report_file, "w") as f:
            f.write("=" * 80 + "\n")
            f.write(f"VerusAgent Statistics Report - {self.benchmark_name}\n")
            f.write("=" * 80 + "\n\n")

            # Execution Summary
            f.write("EXECUTION SUMMARY\n")
            f.write("-" * 80 + "\n")
            f.write(f"Benchmark: {self.benchmark_name}\n")
            f.write(f"Start Time: {self.stats['start_time']}\n")
            f.write(f"End Time: {self.stats.get('end_time', 'N/A')}\n")
            f.write(f"Total Execution Time: {summary['execution_time']:.2f}s\n")
            f.write(f"Verification Success: {'Yes' if summary['verification_success'] else 'No'}\n")
            f.write("\n")

            # Module Activation
            f.write("MODULE ACTIVATION\n")
            f.write("-" * 80 + "\n")
            f.write(f"Total Modules Activated: {summary['modules_activated']}\n")
            f.write("Modules: " + ", ".join(self.stats["modules_activated"]) + "\n")
            f.write("\n")

            # LLM Calls
            f.write("LLM CALLS\n")
            f.write("-" * 80 + "\n")
            f.write(f"Total LLM Calls: {summary['total_llm_calls']}\n")
            f.write(f"Average Response Time: {summary['avg_response_time']:.2f}s\n")
            f.write(f"Cache Hit Rate: {summary['cache_hit_rate']:.2f}%\n")
            f.write("\nCalls by Stage:\n")
            for stage, count in sorted(self.stats["llm_calls"]["by_stage"].items()):
                f.write(f"  {stage}: {count}\n")
            f.write("\n")

            # Iterations
            f.write("ITERATIONS\n")
            f.write("-" * 80 + "\n")
            f.write("Iterations by Module:\n")
            for module, count in sorted(self.stats["iterations"]["by_module"].items()):
                f.write(f"  {module}: {count}\n")
            f.write("\n")

            # Repairs
            f.write("REPAIRS\n")
            f.write("-" * 80 + "\n")
            f.write(f"Total Repair Rounds: {summary['total_repair_rounds']}\n")
            f.write(f"Total Repairs: {summary['total_repairs']}\n")
            f.write(f"Successful Repairs: {self.stats['repairs']['successful_repairs']}\n")
            f.write(f"Failed Repairs: {self.stats['repairs']['failed_repairs']}\n")
            f.write(f"Success Rate: {summary['repair_success_rate']:.2f}%\n")
            f.write("\nRepairs by Error Type:\n")
            for error_type, count in sorted(self.stats["repairs"]["repairs_by_type"].items()):
                f.write(f"  {error_type}: {count}\n")
            f.write("\nRepairs by Heuristic:\n")
            for heuristic, count in sorted(self.stats["repairs"]["repairs_by_heuristic"].items()):
                f.write(f"  {heuristic}: {count}\n")
            f.write("\n")

            # Error Tracking
            f.write("ERROR TRACKING\n")
            f.write("-" * 80 + "\n")
            f.write(f"Initial Errors: {summary['initial_errors']}\n")
            f.write(f"Final Errors: {summary['final_errors']}\n")
            f.write(f"Errors Fixed: {summary['errors_fixed']}\n")
            f.write("\nInitial Errors by Type:\n")
            for error_type, count in sorted(self.stats["errors"]["errors_by_type"].items()):
                f.write(f"  {error_type}: {count}\n")
            f.write("\n")

            # Stage Details
            f.write("STAGE DETAILS\n")
            f.write("-" * 80 + "\n")
            for stage_name, stage_data in sorted(
                self.stats["stages"].items(), key=lambda x: x[1]["step_number"]
            ):
                f.write(f"\n{stage_name} (Step {stage_data['step_number']})\n")
                f.write(f"  Execution Time: {stage_data.get('execution_time', 0):.2f}s\n")
                f.write(f"  LLM Calls: {stage_data['llm_calls']}\n")
                f.write(f"  Iterations: {stage_data['iterations']}\n")
                if stage_data.get("result"):
                    result = stage_data["result"]
                    f.write(f"  Result: Verified={result['verified']}, Errors={result['errors']}\n")

            f.write("\n" + "=" * 80 + "\n")

        self.logger.info(f"Saved human-readable report to {report_file}")
