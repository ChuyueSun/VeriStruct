#!/usr/bin/env python3
"""
Statistics Aggregator for VerusAgent Research Paper

This script aggregates statistics from multiple benchmark runs and generates
comprehensive reports suitable for research paper inclusion, including:
- Summary tables (CSV, LaTeX)
- Detailed analysis
- Performance metrics
- Success rate analysis
"""

import argparse
import json
import statistics as stats
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, List


class StatisticsAggregator:
    """
    Aggregates statistics from multiple benchmark runs.
    """

    def __init__(self, results_dir: Path):
        """
        Initialize the aggregator.

        Args:
            results_dir: Directory containing results from multiple runs
        """
        self.results_dir = results_dir
        self.benchmarks = []
        self.aggregate_stats = defaultdict(list)

    def collect_statistics(self):
        """
        Collect statistics from all benchmark runs.
        """
        print(f"Collecting statistics from {self.results_dir}...")

        # Find all statistics directories
        stats_dirs = list(self.results_dir.rglob("statistics"))

        if not stats_dirs:
            print(f"Warning: No statistics directories found in {self.results_dir}")
            return

        print(f"Found {len(stats_dirs)} statistics directories")

        # Collect from each statistics directory
        for stats_dir in stats_dirs:
            self._collect_from_directory(stats_dir)

        print(f"Collected statistics from {len(self.benchmarks)} benchmarks")

    def _collect_from_directory(self, stats_dir: Path):
        """
        Collect statistics from a single statistics directory.

        Args:
            stats_dir: Path to the statistics directory
        """
        # Find detailed JSON files
        detailed_files = list(stats_dir.glob("detailed_*.json"))

        if not detailed_files:
            return

        # Use the most recent file
        detailed_file = max(detailed_files, key=lambda p: p.stat().st_mtime)

        try:
            with open(detailed_file, "r") as f:
                data = json.load(f)
                self.benchmarks.append(data)

                # Extract key metrics for aggregation
                self._aggregate_metrics(data)

        except Exception as e:
            print(f"Error reading {detailed_file}: {e}")

    def _aggregate_metrics(self, data: Dict[str, Any]):
        """
        Extract and aggregate key metrics from benchmark data.

        Args:
            data: Benchmark statistics data
        """
        # Execution metrics
        self.aggregate_stats["execution_times"].append(
            data.get("total_execution_time", 0)
        )

        # Module activation
        self.aggregate_stats["modules_activated"].append(data.get("modules_count", 0))

        # LLM calls
        llm_calls = data.get("llm_calls", {})
        self.aggregate_stats["total_llm_calls"].append(llm_calls.get("total", 0))
        self.aggregate_stats["cache_hits"].append(llm_calls.get("cache_hits", 0))
        self.aggregate_stats["cache_misses"].append(llm_calls.get("cache_misses", 0))

        # Response times
        response_times = [rt["time"] for rt in llm_calls.get("response_times", [])]
        if response_times:
            self.aggregate_stats["avg_response_times"].append(
                sum(response_times) / len(response_times)
            )

        # Repairs
        repairs = data.get("repairs", {})
        self.aggregate_stats["repair_rounds"].append(repairs.get("total_rounds", 0))
        self.aggregate_stats["total_repairs"].append(repairs.get("total_repairs", 0))
        self.aggregate_stats["successful_repairs"].append(
            repairs.get("successful_repairs", 0)
        )

        # Errors
        errors = data.get("errors", {})
        self.aggregate_stats["initial_errors"].append(
            errors.get("initial_error_count", 0)
        )
        self.aggregate_stats["final_errors"].append(errors.get("final_error_count", 0))

        # Verification success
        verification = data.get("verification", {})
        success = verification.get("final_errors", 1) == 0
        self.aggregate_stats["verification_success"].append(1 if success else 0)

        # Track repair heuristics used
        for heuristic, count in repairs.get("repairs_by_heuristic", {}).items():
            self.aggregate_stats["repair_heuristics"][heuristic] += count

        # Track error types
        for error_type, count in errors.get("errors_by_type", {}).items():
            self.aggregate_stats["error_types"][error_type] += count

        # LLM calls by stage
        for stage, count in llm_calls.get("by_stage", {}).items():
            self.aggregate_stats["llm_by_stage"][stage] += count

    def generate_summary_table(self) -> str:
        """
        Generate a summary table in CSV format.

        Returns:
            CSV formatted summary table
        """
        lines = []

        # Header
        header = [
            "Benchmark",
            "Exec Time (s)",
            "Modules",
            "LLM Calls",
            "Avg Response (s)",
            "Cache Hit %",
            "Repair Rounds",
            "Repairs",
            "Success Rate %",
            "Initial Errors",
            "Final Errors",
            "Verified",
        ]
        lines.append(",".join(header))

        # Data rows
        for benchmark_data in sorted(
            self.benchmarks, key=lambda x: x.get("benchmark_name", "")
        ):
            name = benchmark_data.get("benchmark_name", "unknown")
            exec_time = benchmark_data.get("total_execution_time", 0)
            modules = benchmark_data.get("modules_count", 0)

            llm_calls = benchmark_data.get("llm_calls", {})
            total_llm = llm_calls.get("total", 0)
            cache_hits = llm_calls.get("cache_hits", 0)
            cache_rate = (cache_hits / total_llm * 100) if total_llm > 0 else 0

            response_times = [rt["time"] for rt in llm_calls.get("response_times", [])]
            avg_response = (
                sum(response_times) / len(response_times) if response_times else 0
            )

            repairs = benchmark_data.get("repairs", {})
            repair_rounds = repairs.get("total_rounds", 0)
            total_repairs = repairs.get("total_repairs", 0)
            successful = repairs.get("successful_repairs", 0)
            success_rate = (
                (successful / total_repairs * 100) if total_repairs > 0 else 0
            )

            errors = benchmark_data.get("errors", {})
            initial_errors = errors.get("initial_error_count", 0)
            final_errors = errors.get("final_error_count", 0)

            verification = benchmark_data.get("verification", {})
            verified = "Yes" if verification.get("final_errors", 1) == 0 else "No"

            row = [
                name,
                f"{exec_time:.2f}",
                str(modules),
                str(total_llm),
                f"{avg_response:.2f}",
                f"{cache_rate:.1f}",
                str(repair_rounds),
                str(total_repairs),
                f"{success_rate:.1f}",
                str(initial_errors),
                str(final_errors),
                verified,
            ]
            lines.append(",".join(row))

        return "\n".join(lines)

    def generate_latex_table(self) -> str:
        """
        Generate a LaTeX table suitable for research papers.

        Returns:
            LaTeX formatted table
        """
        lines = []

        lines.append("\\begin{table}[htbp]")
        lines.append("\\centering")
        lines.append("\\caption{VerusAgent Performance on Benchmarks}")
        lines.append("\\label{tab:verus-performance}")
        lines.append("\\begin{tabular}{lrrrrrrr}")
        lines.append("\\toprule")
        lines.append(
            "Benchmark & Time(s) & Modules & LLM Calls & Repairs & Success\\% & Init Err & Final Err \\\\"
        )
        lines.append("\\midrule")

        for benchmark_data in sorted(
            self.benchmarks, key=lambda x: x.get("benchmark_name", "")
        ):
            name = benchmark_data.get("benchmark_name", "unknown").replace("_", "\\_")
            exec_time = benchmark_data.get("total_execution_time", 0)
            modules = benchmark_data.get("modules_count", 0)

            llm_calls = benchmark_data.get("llm_calls", {})
            total_llm = llm_calls.get("total", 0)

            repairs = benchmark_data.get("repairs", {})
            total_repairs = repairs.get("total_repairs", 0)
            successful = repairs.get("successful_repairs", 0)
            success_rate = (
                (successful / total_repairs * 100) if total_repairs > 0 else 0
            )

            errors = benchmark_data.get("errors", {})
            initial_errors = errors.get("initial_error_count", 0)
            final_errors = errors.get("final_error_count", 0)

            row = f"{name} & {exec_time:.1f} & {modules} & {total_llm} & {total_repairs} & {success_rate:.1f} & {initial_errors} & {final_errors} \\\\"
            lines.append(row)

        lines.append("\\bottomrule")
        lines.append("\\end{tabular}")
        lines.append("\\end{table}")

        return "\n".join(lines)

    def generate_aggregate_report(self) -> str:
        """
        Generate an aggregate statistical report.

        Returns:
            Formatted aggregate report
        """
        lines = []

        lines.append("=" * 80)
        lines.append("AGGREGATE STATISTICS REPORT")
        lines.append("=" * 80)
        lines.append("")

        # Overall Summary
        lines.append("OVERALL SUMMARY")
        lines.append("-" * 80)
        lines.append(f"Total Benchmarks: {len(self.benchmarks)}")

        success_count = sum(self.aggregate_stats["verification_success"])
        success_rate = (
            (success_count / len(self.benchmarks) * 100) if self.benchmarks else 0
        )
        lines.append(f"Successfully Verified: {success_count} ({success_rate:.1f}%)")
        lines.append("")

        # Execution Time Statistics
        exec_times = self.aggregate_stats["execution_times"]
        if exec_times:
            lines.append("EXECUTION TIME STATISTICS")
            lines.append("-" * 80)
            lines.append(f"Mean: {stats.mean(exec_times):.2f}s")
            lines.append(f"Median: {stats.median(exec_times):.2f}s")
            lines.append(
                f"Std Dev: {stats.stdev(exec_times):.2f}s"
                if len(exec_times) > 1
                else "Std Dev: N/A"
            )
            lines.append(f"Min: {min(exec_times):.2f}s")
            lines.append(f"Max: {max(exec_times):.2f}s")
            lines.append(f"Total: {sum(exec_times):.2f}s")
            lines.append("")

        # LLM Call Statistics
        llm_calls = self.aggregate_stats["total_llm_calls"]
        if llm_calls:
            lines.append("LLM CALL STATISTICS")
            lines.append("-" * 80)
            lines.append(f"Total LLM Calls: {sum(llm_calls)}")
            lines.append(f"Mean per Benchmark: {stats.mean(llm_calls):.1f}")
            lines.append(f"Median per Benchmark: {stats.median(llm_calls):.1f}")
            lines.append(f"Min: {min(llm_calls)}")
            lines.append(f"Max: {max(llm_calls)}")

            # Cache statistics
            total_cache_hits = sum(self.aggregate_stats["cache_hits"])
            total_cache_misses = sum(self.aggregate_stats["cache_misses"])
            total_calls = total_cache_hits + total_cache_misses
            cache_hit_rate = (
                (total_cache_hits / total_calls * 100) if total_calls > 0 else 0
            )
            lines.append(f"Cache Hit Rate: {cache_hit_rate:.1f}%")
            lines.append("")

        # LLM Calls by Stage
        if self.aggregate_stats["llm_by_stage"]:
            lines.append("LLM CALLS BY STAGE")
            lines.append("-" * 80)
            for stage, count in sorted(
                self.aggregate_stats["llm_by_stage"].items(), key=lambda x: -x[1]
            ):
                lines.append(f"  {stage}: {count}")
            lines.append("")

        # Response Time Statistics
        response_times = self.aggregate_stats["avg_response_times"]
        if response_times:
            lines.append("RESPONSE TIME STATISTICS")
            lines.append("-" * 80)
            lines.append(f"Mean: {stats.mean(response_times):.2f}s")
            lines.append(f"Median: {stats.median(response_times):.2f}s")
            lines.append(f"Min: {min(response_times):.2f}s")
            lines.append(f"Max: {max(response_times):.2f}s")
            lines.append("")

        # Module Activation Statistics
        modules_activated = self.aggregate_stats["modules_activated"]
        if modules_activated:
            lines.append("MODULE ACTIVATION STATISTICS")
            lines.append("-" * 80)
            lines.append(
                f"Mean Modules per Benchmark: {stats.mean(modules_activated):.1f}"
            )
            lines.append(f"Median: {stats.median(modules_activated):.1f}")
            lines.append(f"Min: {min(modules_activated)}")
            lines.append(f"Max: {max(modules_activated)}")
            lines.append("")

        # Repair Statistics
        repair_rounds = self.aggregate_stats["repair_rounds"]
        total_repairs = self.aggregate_stats["total_repairs"]
        successful_repairs = self.aggregate_stats["successful_repairs"]

        if repair_rounds:
            lines.append("REPAIR STATISTICS")
            lines.append("-" * 80)
            lines.append(f"Total Repair Rounds: {sum(repair_rounds)}")
            lines.append(f"Mean Rounds per Benchmark: {stats.mean(repair_rounds):.1f}")
            lines.append(f"Total Repairs Attempted: {sum(total_repairs)}")
            lines.append(f"Successful Repairs: {sum(successful_repairs)}")

            total_rep = sum(total_repairs)
            succ_rep = sum(successful_repairs)
            overall_success = (succ_rep / total_rep * 100) if total_rep > 0 else 0
            lines.append(f"Overall Repair Success Rate: {overall_success:.1f}%")
            lines.append("")

        # Repair Heuristics Usage
        if self.aggregate_stats["repair_heuristics"]:
            lines.append("REPAIR HEURISTICS USAGE")
            lines.append("-" * 80)
            for heuristic, count in sorted(
                self.aggregate_stats["repair_heuristics"].items(), key=lambda x: -x[1]
            ):
                lines.append(f"  {heuristic}: {count}")
            lines.append("")

        # Error Type Distribution
        if self.aggregate_stats["error_types"]:
            lines.append("ERROR TYPE DISTRIBUTION")
            lines.append("-" * 80)
            for error_type, count in sorted(
                self.aggregate_stats["error_types"].items(), key=lambda x: -x[1]
            ):
                lines.append(f"  {error_type}: {count}")
            lines.append("")

        # Error Fixing Statistics
        initial_errors = self.aggregate_stats["initial_errors"]
        final_errors = self.aggregate_stats["final_errors"]

        if initial_errors:
            lines.append("ERROR FIXING STATISTICS")
            lines.append("-" * 80)
            lines.append(f"Total Initial Errors: {sum(initial_errors)}")
            lines.append(f"Total Final Errors: {sum(final_errors)}")
            total_fixed = sum(initial_errors) - sum(final_errors)
            lines.append(f"Total Errors Fixed: {total_fixed}")
            fix_rate = (
                (total_fixed / sum(initial_errors) * 100)
                if sum(initial_errors) > 0
                else 0
            )
            lines.append(f"Error Fix Rate: {fix_rate:.1f}%")
            lines.append("")

        lines.append("=" * 80)

        return "\n".join(lines)

    def save_reports(self, output_dir: Path):
        """
        Save all generated reports.

        Args:
            output_dir: Directory to save reports
        """
        output_dir.mkdir(exist_ok=True, parents=True)

        # CSV summary table
        csv_file = output_dir / "summary_table.csv"
        with open(csv_file, "w") as f:
            f.write(self.generate_summary_table())
        print(f"Saved CSV summary table to {csv_file}")

        # LaTeX table
        latex_file = output_dir / "summary_table.tex"
        with open(latex_file, "w") as f:
            f.write(self.generate_latex_table())
        print(f"Saved LaTeX table to {latex_file}")

        # Aggregate report
        report_file = output_dir / "aggregate_report.txt"
        with open(report_file, "w") as f:
            f.write(self.generate_aggregate_report())
        print(f"Saved aggregate report to {report_file}")

        # Save detailed aggregate statistics as JSON
        json_file = output_dir / "aggregate_statistics.json"
        with open(json_file, "w") as f:
            # Convert defaultdicts to regular dicts
            stats_dict = {
                k: dict(v) if isinstance(v, defaultdict) else v
                for k, v in self.aggregate_stats.items()
            }
            json.dump(stats_dict, f, indent=2)
        print(f"Saved aggregate statistics JSON to {json_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Aggregate statistics from VerusAgent benchmark runs"
    )
    parser.add_argument(
        "--results-dir",
        type=Path,
        default=Path("results"),
        help="Directory containing benchmark results (default: results)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("aggregate_results"),
        help="Directory to save aggregate reports (default: aggregate_results)",
    )

    args = parser.parse_args()

    if not args.results_dir.exists():
        print(f"Error: Results directory {args.results_dir} not found!")
        sys.exit(1)

    # Create aggregator and collect statistics
    aggregator = StatisticsAggregator(args.results_dir)
    aggregator.collect_statistics()

    if not aggregator.benchmarks:
        print("No benchmark statistics found!")
        sys.exit(1)

    # Generate and save reports
    aggregator.save_reports(args.output_dir)

    print(f"\nProcessed {len(aggregator.benchmarks)} benchmarks")
    print(f"Reports saved to {args.output_dir}")


if __name__ == "__main__":
    main()
