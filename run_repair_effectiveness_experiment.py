#!/usr/bin/env python3
"""
Repair Pipeline Effectiveness Experiment

This script runs a comprehensive experiment to evaluate the effectiveness of the
repair pipeline by comparing three configurations:
1. Full pipeline (with all repair modules enabled)
2. Pipeline without repairs (repairs disabled)
3. Baseline mode (single-shot LLM approach)

The experiment collects detailed statistics about:
- Verification success rates
- Repair module activation and effectiveness
- Error reduction rates
- Execution times and LLM usage
- Per-module success rates
"""

import argparse
import glob
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List


class RepairEffectivenessExperiment:
    """Orchestrates the repair effectiveness experiment."""

    def __init__(
        self,
        benchmarks_dir: Path,
        output_base_dir: Path,
        configs: List[str],
        num_repair_rounds: int = 10,
        parallel: bool = True,
    ):
        """
        Initialize the experiment.

        Args:
            benchmarks_dir: Directory containing benchmark files
            output_base_dir: Base directory for all experiment outputs
            configs: List of LLM config names to use
            num_repair_rounds: Number of repair rounds for full pipeline
            parallel: Whether to run benchmarks in parallel
        """
        self.benchmarks_dir = benchmarks_dir
        self.output_base_dir = output_base_dir
        self.configs = configs
        self.num_repair_rounds = num_repair_rounds
        self.parallel = parallel
        self.timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # Create experiment directory with timestamp
        self.experiment_dir = output_base_dir / f"repair_experiment_{self.timestamp}"
        self.experiment_dir.mkdir(exist_ok=True, parents=True)

        # Create subdirectories for each configuration
        self.config_dirs = {
            "full_pipeline": self.experiment_dir / "full_pipeline",
            "no_repairs": self.experiment_dir / "no_repairs",
            "baseline": self.experiment_dir / "baseline",
        }
        for config_dir in self.config_dirs.values():
            config_dir.mkdir(exist_ok=True, parents=True)

        # Experiment metadata
        self.metadata = {
            "timestamp": self.timestamp,
            "start_time": datetime.now().isoformat(),
            "benchmarks_dir": str(benchmarks_dir),
            "num_repair_rounds": num_repair_rounds,
            "configs": configs,
            "parallel": parallel,
            "configurations": {
                "full_pipeline": {
                    "description": "Full VeriStruct pipeline with all repair modules",
                    "num_repair_rounds": num_repair_rounds,
                    "baseline_mode": False,
                },
                "no_repairs": {
                    "description": "Pipeline without repair rounds",
                    "num_repair_rounds": 0,
                    "baseline_mode": False,
                },
                "baseline": {
                    "description": "Single-shot baseline approach",
                    "num_repair_rounds": 0,
                    "baseline_mode": True,
                },
            },
        }

        self._save_metadata()

    def _save_metadata(self):
        """Save experiment metadata."""
        metadata_file = self.experiment_dir / "experiment_metadata.json"
        with open(metadata_file, "w") as f:
            json.dump(self.metadata, f, indent=2)
        print(f"✓ Saved experiment metadata to {metadata_file}")

    def discover_benchmarks(self) -> List[Path]:
        """
        Discover all benchmark files to test.

        Returns:
            List of benchmark file paths
        """
        pattern = str(self.benchmarks_dir / "*_todo.rs")
        benchmarks = sorted(glob.glob(pattern))

        if not benchmarks:
            print(f"Warning: No benchmarks found matching {pattern}")
            return []

        print(f"✓ Found {len(benchmarks)} benchmarks:")
        for bench in benchmarks:
            print(f"  - {Path(bench).name}")

        return [Path(b) for b in benchmarks]

    def run_configuration(
        self, config_name: str, benchmark: Path, output_dir: Path, config_settings: Dict
    ) -> Dict:
        """
        Run a single benchmark with a specific configuration.

        Args:
            config_name: Name of the configuration
            benchmark: Path to benchmark file
            output_dir: Output directory for this run
            config_settings: Configuration settings

        Returns:
            Dictionary with execution results
        """
        benchmark_name = benchmark.stem
        bench_output_dir = output_dir / benchmark_name
        bench_output_dir.mkdir(exist_ok=True, parents=True)

        log_file = bench_output_dir / "output.log"

        print(f"  Running {benchmark_name} with {config_name}...")

        # Set up environment
        env = os.environ.copy()

        # Set configuration-specific environment variables
        if config_settings["baseline_mode"]:
            env["VERUS_BASELINE_MODE"] = "1"
        else:
            env["VERUS_BASELINE_MODE"] = "0"

        env["ENABLE_LLM_CACHE"] = "0"  # Disable cache for fair comparison

        # Build command
        cmd = [
            "./run_agent.py",
            "--test-file",
            str(benchmark),
            "--output-dir",
            str(bench_output_dir),
            "--immutable-functions",
            "test",
            "--no-cache-read",
            "--num-repair-rounds",
            str(config_settings["num_repair_rounds"]),
        ]

        # Run the benchmark
        start_time = time.time()
        try:
            with open(log_file, "w") as log:
                proc = subprocess.run(
                    cmd,
                    env=env,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    text=True,
                    timeout=1800,  # 30 minute timeout
                )
            exit_code = proc.returncode
            success = exit_code == 0
            timeout = False
        except subprocess.TimeoutExpired:
            exit_code = -1
            success = False
            timeout = True
            print(f"    ⚠ Timeout after 30 minutes")
        except Exception as e:
            print(f"    ✗ Error: {e}")
            exit_code = -2
            success = False
            timeout = False

        execution_time = time.time() - start_time

        result = {
            "benchmark": benchmark_name,
            "configuration": config_name,
            "success": success,
            "timeout": timeout,
            "exit_code": exit_code,
            "execution_time": execution_time,
            "log_file": str(log_file),
            "output_dir": str(bench_output_dir),
        }

        # Try to extract statistics if available
        stats_files = list(bench_output_dir.glob("statistics/detailed_*.json"))
        if stats_files:
            stats_file = max(stats_files, key=lambda p: p.stat().st_mtime)
            result["statistics_file"] = str(stats_file)
            try:
                with open(stats_file) as f:
                    stats = json.load(f)
                result["statistics"] = stats
            except Exception as e:
                print(f"    ⚠ Could not load statistics: {e}")

        status = "✓" if success else "✗"
        print(f"    {status} Completed in {execution_time:.1f}s")

        return result

    def run_experiment(self, benchmarks: List[Path]) -> Dict:
        """
        Run the complete experiment across all configurations.

        Args:
            benchmarks: List of benchmark files to test

        Returns:
            Dictionary containing all results
        """
        results = {
            "full_pipeline": [],
            "no_repairs": [],
            "baseline": [],
        }

        print(f"\n{'='*80}")
        print(f"Starting Repair Effectiveness Experiment")
        print(f"{'='*80}\n")

        # Run each configuration
        for config_name, output_dir in self.config_dirs.items():
            config_settings = self.metadata["configurations"][config_name]

            print(f"\n{'-'*80}")
            print(f"Configuration: {config_name}")
            print(f"Description: {config_settings['description']}")
            print(f"{'-'*80}\n")

            for benchmark in benchmarks:
                result = self.run_configuration(config_name, benchmark, output_dir, config_settings)
                results[config_name].append(result)

                # Save incremental results
                self._save_results(results)

        print(f"\n{'='*80}")
        print(f"Experiment Completed!")
        print(f"{'='*80}\n")

        return results

    def _save_results(self, results: Dict):
        """
        Save experiment results.

        Args:
            results: Dictionary of results
        """
        results_file = self.experiment_dir / "experiment_results.json"
        with open(results_file, "w") as f:
            json.dump(results, f, indent=2)

    def generate_summary(self, results: Dict):
        """
        Generate a summary of experiment results.

        Args:
            results: Dictionary of results
        """
        summary_file = self.experiment_dir / "experiment_summary.txt"

        with open(summary_file, "w") as f:
            f.write("=" * 80 + "\n")
            f.write("REPAIR EFFECTIVENESS EXPERIMENT SUMMARY\n")
            f.write("=" * 80 + "\n\n")

            f.write(f"Experiment Timestamp: {self.timestamp}\n")
            f.write(f"Experiment Directory: {self.experiment_dir}\n\n")

            # Overall statistics for each configuration
            for config_name in ["full_pipeline", "no_repairs", "baseline"]:
                config_results = results[config_name]

                f.write(f"\n{'-'*80}\n")
                f.write(f"Configuration: {config_name.upper()}\n")
                f.write(f"{'-'*80}\n\n")

                total_benchmarks = len(config_results)
                successful = sum(1 for r in config_results if r.get("success", False))
                timeouts = sum(1 for r in config_results if r.get("timeout", False))
                failed = total_benchmarks - successful - timeouts

                success_rate = (successful / total_benchmarks * 100) if total_benchmarks > 0 else 0

                f.write(f"Total Benchmarks: {total_benchmarks}\n")
                f.write(f"Successful: {successful} ({success_rate:.1f}%)\n")
                f.write(f"Failed: {failed}\n")
                f.write(f"Timeouts: {timeouts}\n\n")

                # Execution time statistics
                exec_times = [r["execution_time"] for r in config_results]
                if exec_times:
                    f.write(f"Execution Time (seconds):\n")
                    f.write(f"  Mean: {sum(exec_times) / len(exec_times):.2f}\n")
                    f.write(f"  Min: {min(exec_times):.2f}\n")
                    f.write(f"  Max: {max(exec_times):.2f}\n")
                    f.write(f"  Total: {sum(exec_times):.2f}\n\n")

                # Detailed statistics if available
                results_with_stats = [r for r in config_results if "statistics" in r]
                if results_with_stats:
                    f.write(f"Detailed Statistics (from {len(results_with_stats)} benchmarks):\n\n")

                    # LLM calls
                    total_llm_calls = sum(
                        r["statistics"]["llm_calls"]["total"] for r in results_with_stats
                    )
                    avg_llm_calls = (
                        total_llm_calls / len(results_with_stats) if results_with_stats else 0
                    )
                    f.write(f"  Total LLM Calls: {total_llm_calls}\n")
                    f.write(f"  Avg LLM Calls per Benchmark: {avg_llm_calls:.1f}\n\n")

                    # Repairs (only for non-baseline)
                    if config_name != "baseline":
                        total_repairs = sum(
                            r["statistics"]["repairs"]["total_repairs"] for r in results_with_stats
                        )
                        successful_repairs = sum(
                            r["statistics"]["repairs"]["successful_repairs"]
                            for r in results_with_stats
                        )
                        repair_success_rate = (
                            (successful_repairs / total_repairs * 100) if total_repairs > 0 else 0
                        )

                        f.write(f"  Total Repairs Attempted: {total_repairs}\n")
                        f.write(f"  Successful Repairs: {successful_repairs}\n")
                        f.write(f"  Repair Success Rate: {repair_success_rate:.1f}%\n\n")

                        # Repair modules used
                        if config_name == "full_pipeline":
                            repair_modules = {}
                            for r in results_with_stats:
                                for module, count in r["statistics"]["repairs"][
                                    "repairs_by_heuristic"
                                ].items():
                                    repair_modules[module] = repair_modules.get(module, 0) + count

                            if repair_modules:
                                f.write(f"  Repair Modules Used:\n")
                                for module, count in sorted(
                                    repair_modules.items(), key=lambda x: -x[1]
                                ):
                                    f.write(f"    {module}: {count} times\n")
                                f.write("\n")

                    # Errors
                    initial_errors = sum(
                        r["statistics"]["errors"]["initial_error_count"] for r in results_with_stats
                    )
                    final_errors = sum(
                        r["statistics"]["errors"]["final_error_count"] for r in results_with_stats
                    )
                    errors_fixed = initial_errors - final_errors
                    error_reduction = (
                        (errors_fixed / initial_errors * 100) if initial_errors > 0 else 0
                    )

                    f.write(f"  Initial Errors: {initial_errors}\n")
                    f.write(f"  Final Errors: {final_errors}\n")
                    f.write(f"  Errors Fixed: {errors_fixed}\n")
                    f.write(f"  Error Reduction Rate: {error_reduction:.1f}%\n\n")

            f.write("=" * 80 + "\n")

        print(f"✓ Saved experiment summary to {summary_file}")

        # Also print to console
        with open(summary_file) as f:
            print(f.read())


def main():
    parser = argparse.ArgumentParser(description="Run repair pipeline effectiveness experiment")
    parser.add_argument(
        "--benchmarks-dir",
        type=Path,
        default=Path("benchmarks-complete"),
        help="Directory containing benchmark files (default: benchmarks-complete)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("output/repair_experiments"),
        help="Base output directory for experiment results",
    )
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["config-azure"],
        help="LLM configuration names to use",
    )
    parser.add_argument(
        "--num-repair-rounds",
        type=int,
        default=10,
        help="Number of repair rounds for full pipeline (default: 10)",
    )
    parser.add_argument(
        "--benchmark",
        help="Run only a specific benchmark by name (for testing)",
    )
    parser.add_argument(
        "--parallel",
        action="store_true",
        help="Run benchmarks in parallel (not yet implemented)",
    )

    args = parser.parse_args()

    # Validate benchmarks directory
    if not args.benchmarks_dir.exists():
        print(f"Error: Benchmarks directory not found: {args.benchmarks_dir}")
        sys.exit(1)

    # Create experiment
    experiment = RepairEffectivenessExperiment(
        benchmarks_dir=args.benchmarks_dir,
        output_base_dir=args.output_dir,
        configs=args.configs,
        num_repair_rounds=args.num_repair_rounds,
        parallel=args.parallel,
    )

    # Discover benchmarks
    if args.benchmark:
        benchmark_path = args.benchmarks_dir / f"{args.benchmark}.rs"
        if not benchmark_path.exists():
            print(f"Error: Benchmark not found: {benchmark_path}")
            sys.exit(1)
        benchmarks = [benchmark_path]
        print(f"Running single benchmark: {args.benchmark}")
    else:
        benchmarks = experiment.discover_benchmarks()

    if not benchmarks:
        print("Error: No benchmarks to run!")
        sys.exit(1)

    # Run experiment
    results = experiment.run_experiment(benchmarks)

    # Generate summary
    experiment.generate_summary(results)

    print(f"\n✓ Experiment complete! Results saved to:")
    print(f"  {experiment.experiment_dir}")


if __name__ == "__main__":
    main()
