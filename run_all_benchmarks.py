#!/usr/bin/env python3
"""
Script to run all TODO benchmarks in parallel.
Launches one VerusAgent process for each benchmark file.
"""

import multiprocessing
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

# Get the project root directory
PROJECT_ROOT = Path(__file__).parent.absolute()
BENCHMARKS_DIR = PROJECT_ROOT / "benchmarks-complete"

# List of all TODO benchmarks
BENCHMARKS = [
    "atomics_todo.rs",
    "bitmap_2_todo.rs",
    "bitmap_todo.rs",
    "bst_map_todo.rs",
    "invariants_todo.rs",
    "node_todo.rs",
    "option_todo.rs",
    "rb_type_invariant_todo.rs",
    "rwlock_vstd_todo.rs",
    "set_from_vec_todo.rs",
    "transfer_todo.rs",
    "treemap_todo.rs",
    "vectors_todo.rs",
]


def run_benchmark(benchmark_file):
    """Run a single benchmark file."""
    benchmark_path = BENCHMARKS_DIR / benchmark_file
    benchmark_name = benchmark_file.replace(".rs", "")

    print(f"[{benchmark_name}] Starting...")
    start_time = time.time()

    # Set up environment variables
    env = os.environ.copy()
    env["VERUS_TEST_FILE"] = str(benchmark_path)
    env["VERUS_CONFIG"] = "config-azure"

    # Create log file for this benchmark
    log_dir = PROJECT_ROOT / "logs"
    log_dir.mkdir(exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    log_file = log_dir / f"{benchmark_name}_{timestamp}.log"

    try:
        # Run main.py with the benchmark
        with open(log_file, "w") as f:
            process = subprocess.run(
                [sys.executable, "-m", "src.main"],
                cwd=PROJECT_ROOT,
                env=env,
                stdout=f,
                stderr=subprocess.STDOUT,
                timeout=7200,  # 2 hour timeout per benchmark
            )

        elapsed = time.time() - start_time

        if process.returncode == 0:
            print(f"[{benchmark_name}] ✅ COMPLETED in {elapsed:.1f}s - Log: {log_file}")
            return (benchmark_name, "SUCCESS", elapsed, log_file)
        else:
            print(
                f"[{benchmark_name}] ❌ FAILED (exit code {process.returncode}) in {elapsed:.1f}s - Log: {log_file}"
            )
            return (benchmark_name, "FAILED", elapsed, log_file)

    except subprocess.TimeoutExpired:
        elapsed = time.time() - start_time
        print(f"[{benchmark_name}] ⏱️ TIMEOUT after {elapsed:.1f}s - Log: {log_file}")
        return (benchmark_name, "TIMEOUT", elapsed, log_file)
    except Exception as e:
        elapsed = time.time() - start_time
        print(f"[{benchmark_name}] ❌ ERROR: {e} - Log: {log_file}")
        return (benchmark_name, "ERROR", elapsed, log_file)


def main():
    """Main function to run all benchmarks in parallel."""
    print("=" * 80)
    print("VERUSAGENT PARALLEL BENCHMARK RUN")
    print("=" * 80)
    print(f"Total benchmarks: {len(BENCHMARKS)}")
    print(f"Project root: {PROJECT_ROOT}")
    print(f"Benchmarks dir: {BENCHMARKS_DIR}")
    print(f"Start time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # Determine number of parallel workers
    # Use half of available CPUs to avoid overwhelming the system
    num_workers = max(1, multiprocessing.cpu_count() // 2)
    print(f"Parallel workers: {num_workers}")
    print("=" * 80)
    print()

    # Run benchmarks in parallel
    overall_start = time.time()

    with multiprocessing.Pool(processes=num_workers) as pool:
        results = pool.map(run_benchmark, BENCHMARKS)

    overall_elapsed = time.time() - overall_start

    # Print summary
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)

    success_count = sum(1 for _, status, _, _ in results if status == "SUCCESS")
    failed_count = sum(1 for _, status, _, _ in results if status == "FAILED")
    timeout_count = sum(1 for _, status, _, _ in results if status == "TIMEOUT")
    error_count = sum(1 for _, status, _, _ in results if status == "ERROR")

    print(f"Total: {len(results)}")
    print(f"✅ Success: {success_count}")
    print(f"❌ Failed: {failed_count}")
    print(f"⏱️ Timeout: {timeout_count}")
    print(f"❌ Error: {error_count}")
    print(f"Total time: {overall_elapsed:.1f}s ({overall_elapsed/60:.1f}min)")
    print()

    # Print detailed results
    print("DETAILED RESULTS:")
    print("-" * 80)
    for name, status, elapsed, log_file in sorted(results):
        status_icon = {"SUCCESS": "✅", "FAILED": "❌", "TIMEOUT": "⏱️", "ERROR": "❌"}[status]
        print(f"{status_icon} {name:30s} {status:10s} {elapsed:8.1f}s  {log_file}")
    print("=" * 80)

    # Create summary file
    summary_file = (
        PROJECT_ROOT / f"benchmark_summary_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"
    )
    with open(summary_file, "w") as f:
        f.write("VERUSAGENT PARALLEL BENCHMARK RUN SUMMARY\n")
        f.write("=" * 80 + "\n")
        f.write(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"Total: {len(results)}\n")
        f.write(f"Success: {success_count}\n")
        f.write(f"Failed: {failed_count}\n")
        f.write(f"Timeout: {timeout_count}\n")
        f.write(f"Error: {error_count}\n")
        f.write(f"Total time: {overall_elapsed:.1f}s\n")
        f.write("\nDETAILED RESULTS:\n")
        f.write("-" * 80 + "\n")
        for name, status, elapsed, log_file in sorted(results):
            f.write(f"{name:30s} {status:10s} {elapsed:8.1f}s  {log_file}\n")

    print(f"\nSummary saved to: {summary_file}")

    # Check outputs directory
    output_dir = PROJECT_ROOT / "output"
    if output_dir.exists():
        print(f"\nCheck individual benchmark outputs in: {output_dir}")

    # Exit with appropriate code
    if success_count == len(results):
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\nInterrupted by user!")
        sys.exit(130)
