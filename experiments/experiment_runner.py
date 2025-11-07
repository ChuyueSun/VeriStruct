#!/usr/bin/env python3
"""
Automated experiment runner for VeriStruct workflow testing.
Implements the experiment plan defined in EXPERIMENT_PLAN.md
"""

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List

# Add parent directory to path to import VeriStruct modules
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.context import Context
from src.modules.veval import VEval


class ExperimentMetricsCollector:
    """Collects comprehensive metrics for experimental evaluation"""

    def __init__(self, experiment_name: str, output_dir: Path):
        self.experiment_name = experiment_name
        self.output_dir = output_dir
        self.results = []

        # Ensure output directory exists
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def collect_run_metrics(
        self,
        benchmark_name: str,
        context: Context,
        start_time: float,
        end_time: float,
        category: str = "unknown",
    ) -> Dict[str, Any]:
        """Collect all metrics for a single benchmark run"""

        # Calculate basic timing
        elapsed_seconds = end_time - start_time

        # Get final trial evaluation
        final_trial = context.trials[-1] if context.trials else None
        initial_trial = context.trials[0] if context.trials else None

        if not final_trial:
            return self._create_failed_run_metrics(benchmark_name, category, elapsed_seconds)

        final_eval = final_trial.eval
        initial_eval = initial_trial.eval if initial_trial else None

        # Robustness metrics
        robustness = {
            "success": not final_eval.compilation_error and final_eval.errors == 0,
            "modules_completed": self._count_completed_modules(context),
            "errors_encountered": len(final_eval.verus_errors) if final_eval.verus_errors else 0,
            "errors_repaired": self._count_repaired_errors(context),
            "safety_checks_passed": self._count_safety_checks(context, passed=True),
            "safety_checks_failed": self._count_safety_checks(context, passed=False),
            "compilation_success": not final_eval.compilation_error,
            "verification_success": final_eval.errors == 0,
        }

        # Cost metrics
        cost = {
            "total_tokens": self._sum_tokens(context),
            "input_tokens": self._sum_input_tokens(context),
            "output_tokens": self._sum_output_tokens(context),
            "api_calls": self._count_api_calls(context),
            "cache_hits": self._count_cache_hits(context),
            "cache_misses": self._count_cache_misses(context),
            "time_seconds": elapsed_seconds,
            "estimated_cost_usd": self._calculate_cost(context),
        }

        cost["cache_hit_rate"] = (
            cost["cache_hits"] / max(cost["api_calls"], 1) if cost["api_calls"] > 0 else 0.0
        )

        # Effectiveness metrics
        initial_errors = (
            len(initial_eval.verus_errors) if initial_eval and initial_eval.verus_errors else 0
        )
        final_errors = len(final_eval.verus_errors) if final_eval.verus_errors else 0

        effectiveness = {
            "initial_errors": initial_errors,
            "final_errors": final_errors,
            "errors_reduced": initial_errors - final_errors,
            "improvement_rate": (
                (initial_errors - final_errors) / max(initial_errors, 1)
                if initial_errors > 0
                else 0.0
            ),
            "verification_success": final_eval.errors == 0,
            "verified_functions": final_eval.verified if hasattr(final_eval, "verified") else 0,
            "veval_score": {
                "compilation_error": final_eval.compilation_error,
                "verified": final_eval.verified if hasattr(final_eval, "verified") else 0,
                "errors": final_eval.errors,
                "verus_errors": len(final_eval.verus_errors) if final_eval.verus_errors else 0,
            },
        }

        # Module breakdown
        module_breakdown = self._collect_module_metrics(context)

        return {
            "experiment_id": self.experiment_name,
            "benchmark": benchmark_name,
            "category": category,
            "timestamp": datetime.now().isoformat(),
            "robustness": robustness,
            "cost": cost,
            "effectiveness": effectiveness,
            "module_breakdown": module_breakdown,
        }

    def _create_failed_run_metrics(
        self, benchmark_name: str, category: str, elapsed_seconds: float
    ):
        """Create metrics for a failed run"""
        return {
            "experiment_id": self.experiment_name,
            "benchmark": benchmark_name,
            "category": category,
            "timestamp": datetime.now().isoformat(),
            "robustness": {"success": False, "fatal_error": True},
            "cost": {"time_seconds": elapsed_seconds},
            "effectiveness": {"verification_success": False},
        }

    def _count_completed_modules(self, context: Context) -> int:
        """Count how many workflow modules completed successfully"""
        # This would need to be tracked in the Context object
        # For now, estimate based on trials
        return len(context.trials)

    def _count_repaired_errors(self, context: Context) -> int:
        """Count errors that were successfully repaired"""
        if len(context.trials) < 2:
            return 0

        initial_errors = (
            len(context.trials[0].eval.verus_errors) if context.trials[0].eval.verus_errors else 0
        )
        final_errors = (
            len(context.trials[-1].eval.verus_errors) if context.trials[-1].eval.verus_errors else 0
        )

        return max(0, initial_errors - final_errors)

    def _count_safety_checks(self, context: Context, passed: bool) -> int:
        """Count safety checks passed/failed"""
        # Would need to be tracked in Context - placeholder
        return 0

    def _sum_tokens(self, context: Context) -> int:
        """Sum all tokens used"""
        if not hasattr(context, "llm_usage_log"):
            return 0

        total = 0
        for entry in context.llm_usage_log:
            if isinstance(entry, dict) and "usage" in entry:
                usage = entry["usage"]
                total += usage.get("total_tokens", 0)
        return total

    def _sum_input_tokens(self, context: Context) -> int:
        """Sum input tokens"""
        if not hasattr(context, "llm_usage_log"):
            return 0

        total = 0
        for entry in context.llm_usage_log:
            if isinstance(entry, dict) and "usage" in entry:
                usage = entry["usage"]
                total += usage.get("prompt_tokens", 0)
        return total

    def _sum_output_tokens(self, context: Context) -> int:
        """Sum output tokens"""
        if not hasattr(context, "llm_usage_log"):
            return 0

        total = 0
        for entry in context.llm_usage_log:
            if isinstance(entry, dict) and "usage" in entry:
                usage = entry["usage"]
                total += usage.get("completion_tokens", 0)
        return total

    def _count_api_calls(self, context: Context) -> int:
        """Count LLM API calls"""
        if not hasattr(context, "llm_usage_log"):
            return 0
        return len(context.llm_usage_log)

    def _count_cache_hits(self, context: Context) -> int:
        """Count cache hits"""
        if not hasattr(context, "llm_usage_log"):
            return 0

        hits = 0
        for entry in context.llm_usage_log:
            if isinstance(entry, dict) and entry.get("cache_hit", False):
                hits += 1
        return hits

    def _count_cache_misses(self, context: Context) -> int:
        """Count cache misses"""
        return self._count_api_calls(context) - self._count_cache_hits(context)

    def _calculate_cost(self, context: Context) -> float:
        """Calculate estimated USD cost based on token usage"""
        # GPT-4 pricing (approximate)
        INPUT_COST_PER_1K = 0.03
        OUTPUT_COST_PER_1K = 0.06

        input_tokens = self._sum_input_tokens(context)
        output_tokens = self._sum_output_tokens(context)

        cost = input_tokens / 1000 * INPUT_COST_PER_1K + output_tokens / 1000 * OUTPUT_COST_PER_1K

        return round(cost, 4)

    def _collect_module_metrics(self, context: Context) -> Dict[str, Any]:
        """Collect per-module metrics"""
        # Would need detailed tracking in Context
        # Placeholder implementation
        return {}

    def add_result(self, metrics: Dict[str, Any]):
        """Add a result to the collection"""
        self.results.append(metrics)

    def save_results(self):
        """Save collected results to JSON file"""
        output_file = self.output_dir / f"{self.experiment_name}_metrics.json"

        with open(output_file, "w") as f:
            json.dump(self.results, f, indent=2)

        print(f"\n✓ Saved metrics to {output_file}")
        return output_file


class ExperimentRunner:
    """Runs experimental evaluations of VeriStruct workflow"""

    def __init__(self, config_name: str, output_base: Path):
        self.config_name = config_name
        self.output_base = output_base
        self.output_base.mkdir(parents=True, exist_ok=True)

    def load_benchmark_corpus(self, corpus_file: Path) -> List[Dict[str, Any]]:
        """Load benchmark corpus with categories"""
        with open(corpus_file) as f:
            return json.load(f)

    def run_single_benchmark(
        self, benchmark_path: Path, category: str, repair_rounds: int = 5
    ) -> Dict[str, Any]:
        """Run VeriStruct on a single benchmark"""

        print(f"\n{'='*80}")
        print(f"Running benchmark: {benchmark_path.name}")
        print(f"Category: {category}")
        print(f"{'='*80}\n")

        start_time = time.time()

        try:
            # Run VeriStruct
            cmd = [
                sys.executable,
                "run_agent.py",
                "--test-file",
                str(benchmark_path),
                "--config",
                self.config_name,
                "--repair-rounds",
                str(repair_rounds),
                "--output-dir",
                str(self.output_base),
                "--immutable-funcs",
                "test",
            ]

            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=1800  # 30 minute timeout
            )

            end_time = time.time()

            return {
                "success": result.returncode == 0,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "start_time": start_time,
                "end_time": end_time,
                "returncode": result.returncode,
            }

        except subprocess.TimeoutExpired:
            end_time = time.time()
            print(f"✗ Benchmark timed out after 30 minutes")
            return {
                "success": False,
                "timeout": True,
                "start_time": start_time,
                "end_time": end_time,
            }
        except Exception as e:
            end_time = time.time()
            print(f"✗ Error running benchmark: {e}")
            return {
                "success": False,
                "error": str(e),
                "start_time": start_time,
                "end_time": end_time,
            }

    def run_experiment(
        self,
        benchmarks: List[Dict[str, Any]],
        experiment_name: str,
        repair_rounds: int = 5,
    ):
        """Run full experiment on benchmark corpus"""

        output_dir = self.output_base / experiment_name
        output_dir.mkdir(parents=True, exist_ok=True)

        collector = ExperimentMetricsCollector(experiment_name, output_dir)

        total = len(benchmarks)
        successful = 0
        failed = 0

        print(f"\n{'='*80}")
        print(f"EXPERIMENT: {experiment_name}")
        print(f"Total benchmarks: {total}")
        print(f"Output directory: {output_dir}")
        print(f"{'='*80}\n")

        for i, benchmark in enumerate(benchmarks, 1):
            benchmark_path = Path(benchmark["path"])
            category = benchmark["category"]

            print(f"\n[{i}/{total}] Processing: {benchmark_path.name}")

            # Run benchmark
            result = self.run_single_benchmark(benchmark_path, category, repair_rounds)

            # For now, create simplified metrics without Context object
            # In real implementation, would parse output or integrate more deeply
            metrics = {
                "experiment_id": experiment_name,
                "benchmark": benchmark_path.name,
                "category": category,
                "timestamp": datetime.now().isoformat(),
                "robustness": {
                    "success": result.get("success", False),
                    "timeout": result.get("timeout", False),
                },
                "cost": {"time_seconds": result["end_time"] - result["start_time"]},
                "returncode": result.get("returncode", -1),
            }

            collector.add_result(metrics)

            if result.get("success"):
                successful += 1
                print(f"✓ Completed successfully")
            else:
                failed += 1
                print(f"✗ Failed")

        # Save results
        output_file = collector.save_results()

        # Print summary
        print(f"\n{'='*80}")
        print(f"EXPERIMENT COMPLETE: {experiment_name}")
        print(f"{'='*80}")
        print(f"Total: {total}")
        print(f"Successful: {successful} ({successful/total*100:.1f}%)")
        print(f"Failed: {failed} ({failed/total*100:.1f}%)")
        print(f"\nResults saved to: {output_file}")
        print(f"{'='*80}\n")


def main():
    parser = argparse.ArgumentParser(
        description="Run VeriStruct experiments with comprehensive metrics collection"
    )

    parser.add_argument(
        "--corpus", type=Path, required=True, help="Path to benchmark corpus JSON file"
    )

    parser.add_argument("--experiment-name", type=str, required=True, help="Name of the experiment")

    parser.add_argument("--config", type=str, default="config-azure", help="Config name to use")

    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("experiments/results"),
        help="Base output directory for results",
    )

    parser.add_argument("--repair-rounds", type=int, default=5, help="Number of repair rounds")

    parser.add_argument("--limit", type=int, help="Limit number of benchmarks to run (for testing)")

    args = parser.parse_args()

    # Load benchmark corpus
    with open(args.corpus) as f:
        corpus = json.load(f)

    benchmarks = corpus["benchmarks"]

    if args.limit:
        benchmarks = benchmarks[: args.limit]
        print(f"Limiting to {args.limit} benchmarks for testing")

    # Run experiment
    runner = ExperimentRunner(args.config, args.output_dir)
    runner.run_experiment(benchmarks, args.experiment_name, args.repair_rounds)


if __name__ == "__main__":
    main()
