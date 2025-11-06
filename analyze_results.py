#!/usr/bin/env python3
"""
Analyze results from parallel benchmark run.
Checks each benchmark's output for success/failure.
"""

import os
import re
from datetime import datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.absolute()
OUTPUT_DIR = PROJECT_ROOT / "output"

BENCHMARKS = [
    "atomics_todo",
    "bitmap_2_todo",
    "bitmap_todo",
    "bst_map_todo",
    "invariants_todo",
    "node_todo",
    "option_todo",
    "rb_type_invariant_todo",
    "rwlock_vstd_todo",
    "set_from_vec_todo",
    "transfer_todo",
    "treemap_todo",
    "vectors_todo",
]


def parse_score(text):
    """Extract verification score from result file."""
    # Look for patterns like: Verified: 5, Errors: 0, Verus Errors: 0
    verified = re.search(r"Verified:\s*(-?\d+)", text)
    errors = re.search(r"Errors:\s*(\d+)", text)
    verus_errors = re.search(r"Verus Errors:\s*(\d+)", text)
    compilation_error = "Compilation Error: True" in text

    return {
        "verified": int(verified.group(1)) if verified else -1,
        "errors": int(errors.group(1)) if errors else 999,
        "verus_errors": int(verus_errors.group(1)) if verus_errors else 999,
        "compilation_error": compilation_error,
    }


def analyze_benchmark(benchmark_name):
    """Analyze results for a single benchmark."""
    benchmark_dir = OUTPUT_DIR / benchmark_name

    if not benchmark_dir.exists():
        return {
            "name": benchmark_name,
            "status": "NOT_FOUND",
            "message": "Output directory not found",
        }

    # Find most recent run
    run_dirs = sorted(
        [d for d in benchmark_dir.iterdir() if d.is_dir()],
        key=lambda x: x.stat().st_mtime,
        reverse=True,
    )

    if not run_dirs:
        return {
            "name": benchmark_name,
            "status": "NO_RUNS",
            "message": "No run directories found",
        }

    latest_run = run_dirs[0]

    # Check for final result
    final_result = latest_run / "final_result.rs"
    checkpoint_best = list(latest_run.glob("checkpoint_best_*.rs"))
    best_dir = latest_run / "best"

    result_file = None
    if final_result.exists():
        result_file = final_result
    elif checkpoint_best:
        result_file = checkpoint_best[0]
    elif best_dir.exists():
        best_files = list(best_dir.glob("best_*.rs"))
        if best_files:
            result_file = best_files[0]

    if not result_file:
        return {
            "name": benchmark_name,
            "status": "RUNNING",
            "message": f"Still running: {latest_run.name}",
        }

    # Parse the result
    content = result_file.read_text()
    score = parse_score(content)

    # Determine status
    if score["compilation_error"]:
        status = "COMPILATION_ERROR"
    elif score["verified"] > 0 and score["errors"] == 0 and score["verus_errors"] == 0:
        status = "SUCCESS"
    elif score["errors"] == 0 and score["verus_errors"] == 0:
        status = "PARTIAL"  # No errors but not verified
    else:
        status = "FAILED"

    return {
        "name": benchmark_name,
        "status": status,
        "verified": score["verified"],
        "errors": score["errors"],
        "verus_errors": score["verus_errors"],
        "run_dir": latest_run.name,
        "result_file": str(result_file),
    }


def main():
    """Main analysis function."""
    print("=" * 80)
    print("BENCHMARK RESULTS ANALYSIS")
    print("=" * 80)
    print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Output dir: {OUTPUT_DIR}")
    print()

    results = []
    for benchmark in BENCHMARKS:
        result = analyze_benchmark(benchmark)
        results.append(result)

    # Count by status
    status_counts = {}
    for r in results:
        status = r["status"]
        status_counts[status] = status_counts.get(status, 0) + 1

    # Print summary
    print("SUMMARY:")
    print("-" * 80)
    print(f"Total benchmarks: {len(results)}")
    for status, count in sorted(status_counts.items()):
        icon = {
            "SUCCESS": "✅",
            "PARTIAL": "⚠️",
            "FAILED": "❌",
            "COMPILATION_ERROR": "❌",
            "RUNNING": "🔄",
            "NOT_FOUND": "❓",
            "NO_RUNS": "❓",
        }.get(status, "?")
        print(f"{icon} {status:20s}: {count}")
    print()

    # Print detailed results
    print("DETAILED RESULTS:")
    print("-" * 80)
    print(f"{'Benchmark':<30} {'Status':<20} {'V':<4} {'E':<4} {'VE':<4}")
    print("-" * 80)

    for r in sorted(results, key=lambda x: x["name"]):
        icon = {
            "SUCCESS": "✅",
            "PARTIAL": "⚠️",
            "FAILED": "❌",
            "COMPILATION_ERROR": "❌",
            "RUNNING": "🔄",
            "NOT_FOUND": "❓",
            "NO_RUNS": "❓",
        }.get(r["status"], "?")

        v = r.get("verified", "?")
        e = r.get("errors", "?")
        ve = r.get("verus_errors", "?")

        print(f"{icon} {r['name']:<28} {r['status']:<18} {v:<4} {e:<4} {ve:<4}")

        if r["status"] in ["RUNNING", "NOT_FOUND", "NO_RUNS"]:
            print(f"   → {r.get('message', '')}")

    print("=" * 80)
    print("\nLegend: V=Verified, E=Errors, VE=Verus Errors")

    # Print success rate
    if "SUCCESS" in status_counts:
        success_rate = (status_counts["SUCCESS"] / len(results)) * 100
        print(
            f"\n✅ Success Rate: {success_rate:.1f}% ({status_counts['SUCCESS']}/{len(results)})"
        )


if __name__ == "__main__":
    main()
