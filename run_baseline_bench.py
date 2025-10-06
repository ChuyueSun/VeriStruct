#!/usr/bin/env python3
"""
Script to generate baseline results using single-shot LLM inference.

This script runs the baseline approach (single LLM call for specs + proofs) 
on all benchmarks and creates a results-baseline directory for comparison
with the multi-stage pipeline results.
"""

import os
import sys
import glob
import subprocess
import argparse
import time
import json
from pathlib import Path


def check_config_exists(config_name: str) -> bool:
    """
    Check if the specified config file exists.
    
    Args:
        config_name: Name of the config (e.g., 'config-azure')
        
    Returns:
        True if config exists, False otherwise
    """
    # Check in both root and src/configs directories
    config_paths = [
        Path(f"{config_name}.json"),
        Path(f"src/configs/{config_name}.json")
    ]
    
    for path in config_paths:
        if path.exists():
            return True
    return False


def run_single_baseline(benchmark_path: Path, config_name: str, output_dir: Path, timeout_minutes: int = 15) -> dict:
    """
    Run baseline generation on a single benchmark.
    
    Args:
        benchmark_path: Path to the benchmark _todo.rs file
        config_name: Name of the config to use
        output_dir: Directory to store results
        timeout_minutes: Timeout in minutes
        
    Returns:
        Dictionary with execution statistics
    """
    benchmark_name = benchmark_path.stem  # e.g., "rb_type_invariant_todo"
    
    # Create output directory for this benchmark
    bench_output_dir = output_dir / benchmark_name
    bench_output_dir.mkdir(exist_ok=True, parents=True)
    
    log_file = bench_output_dir / "baseline_output.log"
    
    print(f"Running baseline for {benchmark_name}...")
    
    stats = {
        'benchmark': benchmark_name,
        'success': False,
        'timeout': False,
        'error': None,
        'execution_time': 0,
        'exit_code': None,
        'output_files_count': 0,
        'log_size_bytes': 0
    }
    
    try:
        # Set up environment variables for the baseline run
        env = os.environ.copy()
        env['VERUS_TEST_FILE'] = str(benchmark_path.absolute())
        env['VERUS_CONFIG'] = config_name
        env['VERUS_OUTPUT_DIR'] = str(bench_output_dir.absolute())
        env['VERUS_BASELINE_MODE'] = '1'  # Flag to indicate baseline mode
        
        # Run the main VerusAgent with baseline configuration  
        cmd = [
            sys.executable, "-m", "src.main"
        ]
        
        start_time = time.time()
        
        with open(log_file, 'w') as log:
            result = subprocess.run(
                cmd,
                stdout=log,
                stderr=subprocess.STDOUT,
                text=True,
                env=env,
                timeout=timeout_minutes * 60  # Convert to seconds
            )
        
        elapsed_time = time.time() - start_time
        stats['execution_time'] = elapsed_time
        stats['exit_code'] = result.returncode
        
        # Check log file size
        if log_file.exists():
            stats['log_size_bytes'] = log_file.stat().st_size
        
        # Count output files
        output_files = list(bench_output_dir.glob("*.rs"))
        stats['output_files_count'] = len(output_files)
        
        if result.returncode == 0:
            stats['success'] = True
            print(f"  ✓ Completed in {elapsed_time:.1f}s")
        else:
            stats['success'] = False
            print(f"  ✗ Failed (exit code: {result.returncode}) after {elapsed_time:.1f}s")
            
    except subprocess.TimeoutExpired:
        stats['timeout'] = True
        stats['execution_time'] = timeout_minutes * 60
        print(f"  ✗ Timed out after {timeout_minutes} minutes")
    except Exception as e:
        stats['error'] = str(e)
        print(f"  ✗ Error: {e}")
        
    return stats


def collect_summary_stats(all_stats: list) -> dict:
    """
    Calculate summary statistics from individual benchmark results.
    
    Args:
        all_stats: List of individual benchmark stat dictionaries
        
    Returns:
        Summary statistics dictionary
    """
    total_benchmarks = len(all_stats)
    successful = sum(1 for s in all_stats if s['success'])
    timeouts = sum(1 for s in all_stats if s['timeout'])
    errors = sum(1 for s in all_stats if s['error'])
    
    execution_times = [s['execution_time'] for s in all_stats if s['execution_time'] > 0]
    
    summary = {
        'total_benchmarks': total_benchmarks,
        'successful': successful,
        'failed': total_benchmarks - successful,
        'timeouts': timeouts,
        'errors': errors,
        'success_rate': (successful / total_benchmarks * 100) if total_benchmarks > 0 else 0,
        'total_execution_time': sum(execution_times),
        'average_execution_time': sum(execution_times) / len(execution_times) if execution_times else 0,
        'min_execution_time': min(execution_times) if execution_times else 0,
        'max_execution_time': max(execution_times) if execution_times else 0,
        'total_output_files': sum(s['output_files_count'] for s in all_stats),
        'total_log_size_bytes': sum(s['log_size_bytes'] for s in all_stats),
    }
    
    return summary


def save_statistics(baseline_dir: Path, config_name: str, all_stats: list, summary: dict):
    """
    Save detailed statistics to JSON files.
    
    Args:
        baseline_dir: Base output directory
        config_name: Config name used
        all_stats: List of individual benchmark statistics
        summary: Summary statistics
    """
    stats_dir = baseline_dir / "statistics"
    stats_dir.mkdir(exist_ok=True)
    
    # Save detailed stats
    detailed_stats_file = stats_dir / f"{config_name}_detailed_stats.json"
    with open(detailed_stats_file, 'w') as f:
        json.dump(all_stats, f, indent=2)
    
    # Save summary stats
    summary_stats_file = stats_dir / f"{config_name}_summary_stats.json"
    with open(summary_stats_file, 'w') as f:
        json.dump(summary, f, indent=2)
    
    # Create human-readable report
    report_file = stats_dir / f"{config_name}_report.txt"
    with open(report_file, 'w') as f:
        f.write(f"BASELINE EXECUTION REPORT - {config_name}\n")
        f.write("=" * 50 + "\n\n")
        
        f.write("SUMMARY STATISTICS:\n")
        f.write(f"  Total Benchmarks: {summary['total_benchmarks']}\n")
        f.write(f"  Successful: {summary['successful']}\n") 
        f.write(f"  Failed: {summary['failed']}\n")
        f.write(f"  Timeouts: {summary['timeouts']}\n")
        f.write(f"  Errors: {summary['errors']}\n")
        f.write(f"  Success Rate: {summary['success_rate']:.1f}%\n\n")
        
        f.write("PERFORMANCE STATISTICS:\n")
        f.write(f"  Total Execution Time: {summary['total_execution_time']:.1f}s\n")
        f.write(f"  Average Execution Time: {summary['average_execution_time']:.1f}s\n")
        f.write(f"  Min Execution Time: {summary['min_execution_time']:.1f}s\n")
        f.write(f"  Max Execution Time: {summary['max_execution_time']:.1f}s\n\n")
        
        f.write("OUTPUT STATISTICS:\n")
        f.write(f"  Total Output Files: {summary['total_output_files']}\n")
        f.write(f"  Total Log Size: {summary['total_log_size_bytes']} bytes\n\n")
        
        f.write("INDIVIDUAL BENCHMARK RESULTS:\n")
        f.write("-" * 50 + "\n")
        for stat in all_stats:
            status = "SUCCESS" if stat['success'] else "FAILED"
            if stat['timeout']:
                status = "TIMEOUT"
            elif stat['error']:
                status = f"ERROR: {stat['error']}"
                
            f.write(f"{stat['benchmark']:<30} {status:<15} {stat['execution_time']:.1f}s\n")
    
    print(f"\nStatistics saved to {stats_dir}/")


def main():
    parser = argparse.ArgumentParser(
        description="Generate baseline results using single-shot LLM inference"
    )
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["config-azure"],
        help="Config files to use (without .json extension)"
    )
    parser.add_argument(
        "--output-dir",
        default="results-baseline",
        help="Output directory for baseline results"
    )
    parser.add_argument(
        "--benchmark-dir",
        default="benchmarks-complete",
        help="Directory containing benchmark files"
    )
    parser.add_argument(
        "--pattern",
        default="*_todo.rs",
        help="Pattern for benchmark files"
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=15,
        help="Timeout per benchmark in minutes"
    )
    parser.add_argument(
        "--max-benchmarks",
        type=int,
        default=None,
        help="Maximum number of benchmarks to run (for testing)"
    )
    args = parser.parse_args()

    print("=" * 60)
    print("VERUSAGENT BASELINE BENCHMARK SUITE")
    print("=" * 60)

    # Create baseline results directory
    baseline_dir = Path(args.output_dir)
    if baseline_dir.exists():
        import shutil
        print(f"Removing existing {baseline_dir}")
        shutil.rmtree(baseline_dir)
    
    baseline_dir.mkdir(exist_ok=True)
    print(f"Created baseline results directory: {baseline_dir}")

    # Find benchmark files
    benchmark_pattern = Path(args.benchmark_dir) / args.pattern
    benchmark_files = list(glob.glob(str(benchmark_pattern)))
    
    if not benchmark_files:
        print(f"No benchmark files found matching {benchmark_pattern}")
        return 1
    
    # Limit benchmarks if specified
    if args.max_benchmarks:
        benchmark_files = benchmark_files[:args.max_benchmarks]
    
    print(f"Found {len(benchmark_files)} benchmark files to process")

    for config_name in args.configs:
        print(f"\n{'='*20} CONFIG: {config_name} {'='*20}")
        
        # Check if config exists
        if not check_config_exists(config_name):
            print(f"Config file {config_name}.json not found, skipping...")
            continue
        
        print(f"Using config: {config_name}.json")
        
        # Create config-specific output directory
        config_output_dir = baseline_dir / config_name
        config_output_dir.mkdir(exist_ok=True)

        # Run baseline on each benchmark and collect stats
        all_stats = []
        
        for i, benchmark_file in enumerate(sorted(benchmark_files), 1):
            benchmark_path = Path(benchmark_file)
            
            print(f"\n[{i:2d}/{len(benchmark_files)}]", end=" ")
            
            stats = run_single_baseline(
                benchmark_path, 
                config_name, 
                config_output_dir,
                timeout_minutes=args.timeout
            )
            
            all_stats.append(stats)

        # Calculate and save summary statistics
        summary = collect_summary_stats(all_stats)
        save_statistics(baseline_dir, config_name, all_stats, summary)
        
        # Print summary to console
        print(f"\n{'-'*50}")
        print(f"CONFIG {config_name} SUMMARY:")
        print(f"  Success Rate: {summary['success_rate']:.1f}% ({summary['successful']}/{summary['total_benchmarks']})")
        print(f"  Total Time: {summary['total_execution_time']:.1f}s")
        print(f"  Average Time: {summary['average_execution_time']:.1f}s per benchmark")
        print(f"  Timeouts: {summary['timeouts']}")
        print(f"  Errors: {summary['errors']}")

    # Final summary across all configs
    print(f"\n{'='*60}")
    print("BASELINE BENCHMARK COMPLETE")
    print(f"Results saved to: {baseline_dir.absolute()}")
    print(f"Check {baseline_dir}/statistics/ for detailed analysis")
    print("=" * 60)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())