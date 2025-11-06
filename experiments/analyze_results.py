#!/usr/bin/env python3
"""
Statistical analysis and visualization for VerusAgent experiments.
Implements analysis methodology from EXPERIMENT_PLAN.md
"""

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from scipy import stats


class ExperimentAnalyzer:
    """Analyzes experimental results and generates reports"""

    def __init__(self, metrics_file: Path, output_dir: Path):
        self.metrics_file = metrics_file
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Load data
        with open(metrics_file) as f:
            data = json.load(f)

        self.df = pd.DataFrame(data)
        self.results = {}

    def analyze_robustness(self) -> Dict[str, Any]:
        """Analyze robustness metrics"""

        df = self.df

        # Extract robustness columns
        success_col = df["robustness"].apply(lambda x: x.get("success", False))

        n = len(df)
        success_count = success_col.sum()
        success_rate = success_count / n if n > 0 else 0

        # 95% confidence interval for proportion
        if n > 0:
            ci_low, ci_high = stats.binom.interval(0.95, n, success_rate)
            ci_low /= n
            ci_high /= n
        else:
            ci_low, ci_high = 0, 0

        results = {
            "total_runs": n,
            "successful_runs": int(success_count),
            "failed_runs": n - int(success_count),
            "success_rate": success_rate,
            "success_rate_percent": success_rate * 100,
            "confidence_interval_95": {"lower": ci_low, "upper": ci_high},
        }

        # Timeout analysis
        timeout_col = df["robustness"].apply(lambda x: x.get("timeout", False))
        results["timeout_count"] = int(timeout_col.sum())
        results["timeout_rate"] = timeout_col.sum() / n if n > 0 else 0

        # Success by category
        if "category" in df.columns:
            category_success = df.groupby("category").apply(
                lambda g: g["robustness"].apply(lambda x: x.get("success", False)).mean()
            )
            results["success_by_category"] = category_success.to_dict()

        # Compilation vs verification success
        compilation_success = (
            df["robustness"].apply(lambda x: x.get("compilation_success", False)).mean()
        )
        verification_success = (
            df["robustness"].apply(lambda x: x.get("verification_success", False)).mean()
        )

        results["compilation_success_rate"] = compilation_success
        results["verification_success_rate"] = verification_success

        return results

    def analyze_cost(self) -> Dict[str, Any]:
        """Analyze cost metrics"""

        df = self.df

        # Extract cost data
        time_data = df["cost"].apply(lambda x: x.get("time_seconds", 0))
        token_data = df["cost"].apply(lambda x: x.get("total_tokens", 0))
        cost_data = df["cost"].apply(lambda x: x.get("estimated_cost_usd", 0))
        cache_hit_rate = df["cost"].apply(lambda x: x.get("cache_hit_rate", 0))

        results = {
            "time": {
                "mean_seconds": time_data.mean(),
                "median_seconds": time_data.median(),
                "std_seconds": time_data.std(),
                "mean_minutes": time_data.mean() / 60,
                "total_hours": time_data.sum() / 3600,
            },
            "tokens": {
                "mean": token_data.mean(),
                "median": token_data.median(),
                "std": token_data.std(),
                "total": token_data.sum(),
                "min": token_data.min(),
                "max": token_data.max(),
            },
            "cost_usd": {
                "mean": cost_data.mean(),
                "median": cost_data.median(),
                "std": cost_data.std(),
                "total": cost_data.sum(),
                "min": cost_data.min(),
                "max": cost_data.max(),
            },
            "cache": {
                "mean_hit_rate": cache_hit_rate.mean(),
                "median_hit_rate": cache_hit_rate.median(),
            },
        }

        # Cost by category
        if "category" in df.columns:
            category_cost = df.groupby("category").apply(
                lambda g: g["cost"].apply(lambda x: x.get("estimated_cost_usd", 0)).mean()
            )
            results["cost_by_category"] = category_cost.to_dict()

        return results

    def analyze_effectiveness(self) -> Dict[str, Any]:
        """Analyze effectiveness metrics"""

        df = self.df

        # Filter out runs that don't have effectiveness data
        has_effectiveness = df["effectiveness"].apply(lambda x: isinstance(x, dict))
        df_valid = df[has_effectiveness]

        if len(df_valid) == 0:
            return {"error": "No valid effectiveness data"}

        # Extract effectiveness data
        verification_success = df_valid["effectiveness"].apply(
            lambda x: x.get("verification_success", False)
        )

        improvement_rate = df_valid["effectiveness"].apply(lambda x: x.get("improvement_rate", 0))

        errors_reduced = df_valid["effectiveness"].apply(lambda x: x.get("errors_reduced", 0))

        results = {
            "verification_success_rate": verification_success.mean(),
            "verification_success_count": int(verification_success.sum()),
            "total_benchmarks": len(df_valid),
            "improvement": {
                "mean_rate": improvement_rate.mean(),
                "median_rate": improvement_rate.median(),
                "std_rate": improvement_rate.std(),
            },
            "errors_reduced": {
                "mean": errors_reduced.mean(),
                "median": errors_reduced.median(),
                "total": errors_reduced.sum(),
            },
        }

        return results

    def generate_visualizations(self):
        """Generate visualization plots"""

        df = self.df

        # Set style
        sns.set_style("whitegrid")
        plt.rcParams["figure.figsize"] = (10, 6)

        # 1. Success rate by category
        if "category" in df.columns:
            plt.figure()
            success_by_cat = df.groupby("category").apply(
                lambda g: g["robustness"].apply(lambda x: x.get("success", False)).mean() * 100
            )
            success_by_cat.plot(kind="bar", color="steelblue")
            plt.title("Success Rate by Benchmark Category", fontsize=14, fontweight="bold")
            plt.ylabel("Success Rate (%)")
            plt.xlabel("Category")
            plt.xticks(rotation=45, ha="right")
            plt.ylim(0, 100)
            plt.tight_layout()
            plt.savefig(self.output_dir / "success_by_category.png", dpi=300)
            plt.close()

        # 2. Cost distribution
        plt.figure()
        cost_data = df["cost"].apply(lambda x: x.get("estimated_cost_usd", 0))
        cost_data[cost_data > 0].hist(bins=30, color="coral", edgecolor="black")
        plt.title("Cost Distribution per Benchmark", fontsize=14, fontweight="bold")
        plt.xlabel("Cost (USD)")
        plt.ylabel("Frequency")
        plt.tight_layout()
        plt.savefig(self.output_dir / "cost_distribution.png", dpi=300)
        plt.close()

        # 3. Time distribution
        plt.figure()
        time_data = df["cost"].apply(lambda x: x.get("time_seconds", 0) / 60)
        time_data[time_data > 0].hist(bins=30, color="lightgreen", edgecolor="black")
        plt.title("Execution Time Distribution", fontsize=14, fontweight="bold")
        plt.xlabel("Time (minutes)")
        plt.ylabel("Frequency")
        plt.tight_layout()
        plt.savefig(self.output_dir / "time_distribution.png", dpi=300)
        plt.close()

        # 4. Tokens vs Time scatter
        plt.figure()
        tokens = df["cost"].apply(lambda x: x.get("total_tokens", 0))
        time_min = df["cost"].apply(lambda x: x.get("time_seconds", 0) / 60)

        # Filter out zero values
        valid_mask = (tokens > 0) & (time_min > 0)
        plt.scatter(tokens[valid_mask], time_min[valid_mask], alpha=0.6, color="purple")
        plt.xlabel("Total Tokens")
        plt.ylabel("Time (minutes)")
        plt.title("Token Usage vs Execution Time", fontsize=14, fontweight="bold")
        plt.tight_layout()
        plt.savefig(self.output_dir / "tokens_vs_time.png", dpi=300)
        plt.close()

        # 5. Success/Failure pie chart
        plt.figure()
        success_counts = df["robustness"].apply(lambda x: x.get("success", False)).value_counts()
        colors = ["#90EE90", "#FFB6C1"]  # Light green and light red
        plt.pie(
            success_counts.values,
            labels=["Success", "Failure"],
            autopct="%1.1f%%",
            startangle=90,
            colors=colors,
        )
        plt.title("Overall Success Rate", fontsize=14, fontweight="bold")
        plt.tight_layout()
        plt.savefig(self.output_dir / "success_pie_chart.png", dpi=300)
        plt.close()

        print(f"✓ Generated visualizations in {self.output_dir}")

    def generate_report(self) -> str:
        """Generate comprehensive markdown report"""

        robustness = self.analyze_robustness()
        cost = self.analyze_cost()
        effectiveness = self.analyze_effectiveness()

        # Store results
        self.results = {
            "robustness": robustness,
            "cost": cost,
            "effectiveness": effectiveness,
        }

        # Generate markdown report
        report = f"""# VerusAgent Experimental Evaluation Results

**Experiment**: {self.df['experiment_id'].iloc[0] if len(self.df) > 0 else 'Unknown'}
**Date**: {self.df['timestamp'].iloc[0] if len(self.df) > 0 else 'Unknown'}
**Total Benchmarks**: {robustness['total_runs']}

---

## Executive Summary

This report presents the results of a comprehensive experimental evaluation of the VerusAgent workflow,
assessing its **robustness**, **cost-effectiveness**, and **overall effectiveness** in automating
formal verification for Rust/Verus code.

### Key Findings

- **Success Rate**: {robustness['success_rate_percent']:.1f}% ({robustness['successful_runs']}/{robustness['total_runs']} benchmarks)
- **Verification Success**: {effectiveness.get('verification_success_rate', 0)*100:.1f}%
- **Average Cost**: ${cost['cost_usd']['mean']:.2f} per benchmark
- **Average Time**: {cost['time']['mean_minutes']:.1f} minutes per benchmark
- **Total Experiment Cost**: ${cost['cost_usd']['total']:.2f}

---

## 1. Robustness Analysis

### Overall Performance

| Metric | Value |
|--------|-------|
| **Total Runs** | {robustness['total_runs']} |
| **Successful** | {robustness['successful_runs']} ({robustness['success_rate_percent']:.1f}%) |
| **Failed** | {robustness['failed_runs']} ({100-robustness['success_rate_percent']:.1f}%) |
| **Timeouts** | {robustness['timeout_count']} ({robustness['timeout_rate']*100:.1f}%) |
| **95% Confidence Interval** | [{robustness['confidence_interval_95']['lower']*100:.1f}%, {robustness['confidence_interval_95']['upper']*100:.1f}%] |

### Compilation vs Verification

- **Compilation Success Rate**: {robustness.get('compilation_success_rate', 0)*100:.1f}%
- **Verification Success Rate**: {robustness.get('verification_success_rate', 0)*100:.1f}%

### Success Rate by Category

"""

        if "success_by_category" in robustness:
            report += "| Category | Success Rate |\n|----------|-------------|\n"
            for cat, rate in sorted(robustness["success_by_category"].items()):
                report += f"| {cat} | {rate*100:.1f}% |\n"

        report += f"""

![Success by Category](success_by_category.png)

---

## 2. Cost Analysis

### Time Performance

| Metric | Value |
|--------|-------|
| **Mean Time** | {cost['time']['mean_minutes']:.1f} minutes |
| **Median Time** | {cost['time']['median_seconds']/60:.1f} minutes |
| **Std Dev** | {cost['time']['std_seconds']/60:.1f} minutes |
| **Total Time** | {cost['time']['total_hours']:.1f} hours |

### Token Usage

| Metric | Value |
|--------|-------|
| **Mean Tokens** | {cost['tokens']['mean']:,.0f} |
| **Median Tokens** | {cost['tokens']['median']:,.0f} |
| **Total Tokens** | {cost['tokens']['total']:,.0f} |
| **Min Tokens** | {cost['tokens']['min']:,.0f} |
| **Max Tokens** | {cost['tokens']['max']:,.0f} |

### Financial Cost

| Metric | Value |
|--------|-------|
| **Mean Cost** | ${cost['cost_usd']['mean']:.2f} |
| **Median Cost** | ${cost['cost_usd']['median']:.2f} |
| **Total Cost** | ${cost['cost_usd']['total']:.2f} |
| **Min Cost** | ${cost['cost_usd']['min']:.2f} |
| **Max Cost** | ${cost['cost_usd']['max']:.2f} |

### Cache Performance

- **Mean Cache Hit Rate**: {cost['cache']['mean_hit_rate']*100:.1f}%
- **Median Cache Hit Rate**: {cost['cache']['median_hit_rate']*100:.1f}%

![Cost Distribution](cost_distribution.png)

![Time Distribution](time_distribution.png)

![Tokens vs Time](tokens_vs_time.png)

---

## 3. Effectiveness Analysis

"""

        if "error" not in effectiveness:
            report += f"""
### Verification Performance

| Metric | Value |
|--------|-------|
| **Verification Success Rate** | {effectiveness['verification_success_rate']*100:.1f}% |
| **Benchmarks Fully Verified** | {effectiveness['verification_success_count']}/{effectiveness['total_benchmarks']} |

### Error Reduction

| Metric | Value |
|--------|-------|
| **Mean Improvement Rate** | {effectiveness['improvement']['mean_rate']*100:.1f}% |
| **Median Improvement Rate** | {effectiveness['improvement']['median_rate']*100:.1f}% |
| **Mean Errors Reduced** | {effectiveness['errors_reduced']['mean']:.1f} |
| **Total Errors Reduced** | {effectiveness['errors_reduced']['total']} |

"""
        else:
            report += f"**Note**: {effectiveness['error']}\n\n"

        report += f"""
![Overall Success](success_pie_chart.png)

---

## 4. Statistical Significance

### Hypothesis Test: Success Rate

**Null Hypothesis (H₀)**: Success rate ≤ 50% (no better than random)
**Alternative Hypothesis (H₁)**: Success rate > 50%

"""

        # Perform hypothesis test
        n = robustness["total_runs"]
        success_count = robustness["successful_runs"]
        p_value = 1 - stats.binom.cdf(success_count - 1, n, 0.5)

        report += f"""
**Test**: One-sample proportion test
**Result**: p-value = {p_value:.4f}
**Conclusion**: {"✓ REJECT H₀" if p_value < 0.05 else "✗ FAIL TO REJECT H₀"} at α=0.05 significance level

"""

        if p_value < 0.05:
            report += (
                "The success rate is **statistically significantly better than random chance**.\n\n"
            )
        else:
            report += "The success rate is **not statistically significantly better than random chance**.\n\n"

        report += """
---

## 5. Recommendations

Based on the experimental results, we recommend:

"""

        # Generate recommendations based on findings
        if robustness["success_rate"] >= 0.8:
            report += "1. ✓ **Workflow is production-ready** for similar benchmark categories\n"
        elif robustness["success_rate"] >= 0.5:
            report += "1. ⚠ **Workflow shows promise** but needs improvement for production use\n"
        else:
            report += "1. ✗ **Workflow needs significant improvement** before production use\n"

        if cost["cost_usd"]["mean"] < 5:
            report += "2. ✓ **Cost is reasonable** for automation value provided\n"
        else:
            report += "2. ⚠ **Cost optimization recommended** to improve cost-effectiveness\n"

        if cost["cache"]["mean_hit_rate"] < 0.5:
            report += "3. ⚠ **Enable caching** to reduce costs and improve performance\n"

        if "success_by_category" in robustness:
            weak_categories = [
                cat for cat, rate in robustness["success_by_category"].items() if rate < 0.5
            ]
            if weak_categories:
                report += f"4. 🎯 **Focus improvement efforts** on: {', '.join(weak_categories)}\n"

        report += """

---

## Appendix: Raw Data Summary

```json
"""

        report += json.dumps(self.results, indent=2)
        report += "\n```\n"

        return report

    def save_report(self):
        """Save analysis report to file"""
        report = self.generate_report()

        report_file = self.output_dir / "ANALYSIS_REPORT.md"
        with open(report_file, "w") as f:
            f.write(report)

        print(f"✓ Saved analysis report to {report_file}")

        # Also save JSON results
        json_file = self.output_dir / "analysis_results.json"
        with open(json_file, "w") as f:
            json.dump(self.results, f, indent=2)

        print(f"✓ Saved JSON results to {json_file}")

        return report_file


def main():
    parser = argparse.ArgumentParser(description="Analyze VerusAgent experimental results")

    parser.add_argument(
        "--metrics",
        type=Path,
        required=True,
        help="Path to metrics JSON file from experiment runner",
    )

    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("experiments/analysis"),
        help="Output directory for analysis results",
    )

    args = parser.parse_args()

    if not args.metrics.exists():
        print(f"Error: Metrics file not found: {args.metrics}")
        return 1

    # Run analysis
    analyzer = ExperimentAnalyzer(args.metrics, args.output_dir)

    print("\nAnalyzing robustness...")
    robustness = analyzer.analyze_robustness()

    print("Analyzing cost...")
    cost = analyzer.analyze_cost()

    print("Analyzing effectiveness...")
    effectiveness = analyzer.analyze_effectiveness()

    print("\nGenerating visualizations...")
    analyzer.generate_visualizations()

    print("\nGenerating report...")
    analyzer.save_report()

    print("\n" + "=" * 80)
    print("ANALYSIS COMPLETE")
    print("=" * 80)
    print(f"\nResults saved to: {args.output_dir}")
    print(f"View report: {args.output_dir / 'ANALYSIS_REPORT.md'}")
    print("=" * 80 + "\n")

    return 0


if __name__ == "__main__":
    exit(main())
