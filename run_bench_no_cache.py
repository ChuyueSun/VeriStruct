#!/usr/bin/env python3
"""
Run benchmarks with cache disabled for accurate runtime statistics.
"""
import argparse
import glob
import os
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(
        description="Run all *_todo.rs benchmarks with cache disabled for accurate statistics."
    )
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["config-azure"],
        help="List of config file names (without .json) to pass to run_agent.py",
    )
    parser.add_argument(
        "--benchmark",
        help="Run a specific benchmark by name (e.g., 'bst_map_todo' or 'atomics_todo'). If not specified, runs all benchmarks.",
    )
    args = parser.parse_args()

    # Prepare output directory (preserve existing content)
    os.makedirs("output", exist_ok=True)

    # Determine which benchmarks to run
    if args.benchmark:
        # Validate that the benchmark exists
        todo_file = f"benchmarks-complete/{args.benchmark}.rs"
        if not os.path.exists(todo_file):
            print(f"Error: Benchmark '{args.benchmark}' not found. Expected file: {todo_file}")
            print("Available benchmarks:")
            for todo_path in glob.glob("benchmarks-complete/*_todo.rs"):
                name = os.path.splitext(os.path.basename(todo_path))[0]
                print(f"  - {name}")
            return

        benchmarks = [args.benchmark]
        print(f"Running individual benchmark: {args.benchmark}")
    else:
        # Run all benchmarks
        benchmarks = []
        for todo_path in glob.glob("benchmarks-complete/*_todo.rs"):
            name = os.path.splitext(os.path.basename(todo_path))[0]
            benchmarks.append(name)
        print(f"Running all benchmarks: {len(benchmarks)} found")

    for cfg in args.configs:
        cfg_results_root = os.path.join("output", cfg)
        os.makedirs(cfg_results_root, exist_ok=True)

        # Prepare all benchmarks and start them in parallel
        processes = []
        log_files = []

        for benchmark_name in benchmarks:
            test_file = f"benchmarks-complete/{benchmark_name}.rs"

            bench_dir = os.path.join(cfg_results_root, benchmark_name)
            os.makedirs(bench_dir, exist_ok=True)

            log_file = os.path.join(bench_dir, "output.log")
            log_files.append(log_file)

            print(f"Starting {benchmark_name} with {cfg} (cache disabled) -> log: {log_file}")

            # Set environment to disable cache
            env = os.environ.copy()
            env["ENABLE_LLM_CACHE"] = "0"

            cmd = [
                "./run_agent.py",
                "--test-file",
                test_file,
                "--no-cache-read",
                "--output-dir",
                bench_dir,
                "--immutable-functions",
                "test",
                "--num-repair-rounds",
                "10",
            ]

            # Open log file and start process
            log_handle = open(log_file, "w")
            proc = subprocess.Popen(
                cmd,
                env=env,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                text=True,
            )
            processes.append((benchmark_name, proc, log_handle))

        print(f"\n✓ Started {len(processes)} benchmarks in parallel")
        print("Waiting for all benchmarks to complete...\n")

        # Wait for all processes to complete
        for benchmark_name, proc, log_handle in processes:
            proc.wait()
            log_handle.close()
            if proc.returncode == 0:
                print(f"  ✓ Completed {benchmark_name}")
            else:
                print(f"  ✗ Error running {benchmark_name} (exit code: {proc.returncode})")


if __name__ == "__main__":
    main()
