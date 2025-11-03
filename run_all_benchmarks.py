#!/usr/bin/env python3
"""
Script to run all benchmarks from benchmarks-complete directory in parallel.
"""
import argparse
import subprocess
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path


def run_benchmark(benchmark_file, config, verus_path, num_repair_rounds, no_cache_read):
    """Run a single benchmark using run_agent.py"""
    benchmark_name = benchmark_file.stem
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = Path("output") / benchmark_name / timestamp

    cmd = [
        sys.executable,
        "run_agent.py",
        "--test-file",
        str(benchmark_file),
        "--config",
        config,
        "--output-dir",
        str(output_dir),
        "--num-repair-rounds",
        str(num_repair_rounds),
    ]

    if verus_path:
        cmd.extend(["--verus-path", verus_path])

    if no_cache_read:
        cmd.append("--no-cache-read")

    print(f"\n{'='*80}")
    print(f"Starting: {benchmark_name}")
    print(f"Command: {' '.join(cmd)}")
    print(f"Output: {output_dir}")
    print(f"{'='*80}\n")

    start_time = datetime.now()

    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, cwd=Path(__file__).parent
        )

        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()

        # Save output logs
        output_dir.mkdir(parents=True, exist_ok=True)

        with open(output_dir / "stdout.log", "w") as f:
            f.write(result.stdout)

        with open(output_dir / "stderr.log", "w") as f:
            f.write(result.stderr)

        status = "SUCCESS" if result.returncode == 0 else "FAILED"

        return {
            "benchmark": benchmark_name,
            "status": status,
            "returncode": result.returncode,
            "duration": duration,
            "output_dir": str(output_dir),
        }

    except Exception as e:
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()

        return {
            "benchmark": benchmark_name,
            "status": "ERROR",
            "returncode": -1,
            "duration": duration,
            "error": str(e),
            "output_dir": str(output_dir),
        }


def main():
    parser = argparse.ArgumentParser(
        description="Run all benchmarks from benchmarks-complete directory in parallel"
    )
    parser.add_argument(
        "--benchmarks-dir",
        help="Directory containing benchmark files",
        default="benchmarks-complete",
    )
    parser.add_argument(
        "--pattern",
        help="Glob pattern to match benchmark files",
        default="*_todo.rs",
    )
    parser.add_argument(
        "--max-workers",
        type=int,
        help="Maximum number of parallel workers (default: 4)",
        default=4,
    )
    parser.add_argument(
        "--verus-path",
        help="Path to the Verus executable",
        default=None,
    )
    parser.add_argument(
        "--config",
        help="Config file to use (default: config-azure)",
        default="config-azure",
    )
    parser.add_argument(
        "--num-repair-rounds",
        type=int,
        help="Number of repair rounds to run (default: 5)",
        default=5,
    )
    parser.add_argument(
        "--no-cache-read",
        action="store_true",
        help="Disable reading from LLM cache",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be run without actually running",
    )

    args = parser.parse_args()

    # Find all benchmark files
    benchmarks_dir = Path(args.benchmarks_dir)
    if not benchmarks_dir.exists():
        print(f"Error: Benchmarks directory not found: {benchmarks_dir}")
        sys.exit(1)

    benchmark_files = sorted(benchmarks_dir.glob(args.pattern))

    if not benchmark_files:
        print(f"No benchmark files found matching pattern: {args.pattern}")
        sys.exit(1)

    print(f"\n{'='*80}")
    print(f"PARALLEL BENCHMARK RUNNER")
    print(f"{'='*80}")
    print(f"Benchmarks directory: {benchmarks_dir.absolute()}")
    print(f"Pattern: {args.pattern}")
    print(f"Found {len(benchmark_files)} benchmarks:")
    for bf in benchmark_files:
        print(f"  - {bf.name}")
    print(f"Max workers: {args.max_workers}")
    print(f"Config: {args.config}")
    print(f"Repair rounds: {args.num_repair_rounds}")
    print(f"{'='*80}\n")

    if args.dry_run:
        print("DRY RUN - No benchmarks will be executed")
        return

    # Run benchmarks in parallel
    start_time = datetime.now()
    results = []

    with ProcessPoolExecutor(max_workers=args.max_workers) as executor:
        # Submit all tasks
        future_to_benchmark = {
            executor.submit(
                run_benchmark,
                bf,
                args.config,
                args.verus_path,
                args.num_repair_rounds,
                args.no_cache_read,
            ): bf
            for bf in benchmark_files
        }

        # Collect results as they complete
        for future in as_completed(future_to_benchmark):
            benchmark_file = future_to_benchmark[future]
            try:
                result = future.result()
                results.append(result)

                status_symbol = "✓" if result["status"] == "SUCCESS" else "✗"
                print(
                    f"\n{status_symbol} {result['benchmark']}: {result['status']} "
                    f"(took {result['duration']:.2f}s)"
                )

            except Exception as e:
                print(f"\n✗ {benchmark_file.stem}: EXCEPTION - {e}")
                results.append(
                    {
                        "benchmark": benchmark_file.stem,
                        "status": "EXCEPTION",
                        "error": str(e),
                    }
                )

    end_time = datetime.now()
    total_duration = (end_time - start_time).total_seconds()

    # Print summary
    print(f"\n{'='*80}")
    print(f"SUMMARY")
    print(f"{'='*80}")
    print(f"Total time: {total_duration:.2f}s")
    print(f"Total benchmarks: {len(results)}")

    success_count = sum(1 for r in results if r["status"] == "SUCCESS")
    failed_count = sum(
        1 for r in results if r["status"] in ["FAILED", "ERROR", "EXCEPTION"]
    )

    print(f"Successful: {success_count}")
    print(f"Failed: {failed_count}")
    print(f"\nResults by benchmark:")

    for result in sorted(results, key=lambda x: x["benchmark"]):
        status_symbol = "✓" if result["status"] == "SUCCESS" else "✗"
        duration_str = f"{result['duration']:.2f}s" if "duration" in result else "N/A"
        print(
            f"  {status_symbol} {result['benchmark']:30s} {result['status']:10s} {duration_str:>10s}"
        )
        if "output_dir" in result:
            print(f"      Output: {result['output_dir']}")

    print(f"{'='*80}\n")

    # Save summary to file
    summary_file = (
        Path("output")
        / f"benchmark_summary_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"
    )
    summary_file.parent.mkdir(parents=True, exist_ok=True)

    with open(summary_file, "w") as f:
        f.write(f"Benchmark Summary - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"{'='*80}\n")
        f.write(f"Total time: {total_duration:.2f}s\n")
        f.write(f"Total benchmarks: {len(results)}\n")
        f.write(f"Successful: {success_count}\n")
        f.write(f"Failed: {failed_count}\n")
        f.write(f"\nResults:\n")
        for result in sorted(results, key=lambda x: x["benchmark"]):
            status_symbol = "✓" if result["status"] == "SUCCESS" else "✗"
            duration_str = (
                f"{result['duration']:.2f}s" if "duration" in result else "N/A"
            )
            f.write(
                f"  {status_symbol} {result['benchmark']:30s} {result['status']:10s} {duration_str:>10s}\n"
            )
            if "output_dir" in result:
                f.write(f"      Output: {result['output_dir']}\n")

    print(f"Summary saved to: {summary_file}\n")

    sys.exit(0 if failed_count == 0 else 1)


if __name__ == "__main__":
    main()
