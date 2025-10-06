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
from pathlib import Path

# Add src to Python path
sys.path.append(str(Path(__file__).parent / "src"))

def check_config_exists(config_name: str) -> bool:
    """
    Check if the specified config file exists.
    
    Args:
        config_name: Name of the config (e.g., 'config-azure')
        
    Returns:
        True if config exists, False otherwise
    """
    config_path = Path(f"{config_name}.json")
    return config_path.exists()


def run_single_baseline(benchmark_path: Path, config_name: str, output_dir: Path) -> bool:
    """
    Run baseline generation on a single benchmark.
    
    Args:
        benchmark_path: Path to the benchmark _todo.rs file
        config_name: Name of the config to use
        output_dir: Directory to store results
        
    Returns:
        True if successful, False otherwise
    """
    benchmark_name = benchmark_path.stem  # e.g., "rb_type_invariant_todo"
    
    # Create output directory for this benchmark
    bench_output_dir = output_dir / benchmark_name
    bench_output_dir.mkdir(exist_ok=True, parents=True)
    
    log_file = bench_output_dir / "baseline_output.log"
    
    print(f"Running baseline for {benchmark_name}...")
    
    try:
        # Set up environment variables for the baseline run
        env = os.environ.copy()
        env['VERUS_TEST_FILE'] = str(benchmark_path.absolute())
        env['VERUS_CONFIG'] = config_name  # Use the original config
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
                timeout=1800  # 30 minute timeout per benchmark
            )
        
        elapsed_time = time.time() - start_time
        
        if result.returncode == 0:
            print(f"  ✓ Completed in {elapsed_time:.1f}s")
            return True
        else:
            print(f"  ✗ Failed (exit code: {result.returncode}) after {elapsed_time:.1f}s")
            return False
            
    except subprocess.TimeoutExpired:
        print(f"  ✗ Timed out after 30 minutes")
        return False
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return False


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
    args = parser.parse_args()

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
    
    print(f"Found {len(benchmark_files)} benchmark files")

    total_benchmarks = 0
    successful_benchmarks = 0

    for config_name in args.configs:
        print(f"\n=== Running with config: {config_name} ===")
        
        # Create config-specific output directory
        config_output_dir = baseline_dir / config_name
        config_output_dir.mkdir(exist_ok=True)
        
        # Check if config exists
        if not check_config_exists(config_name):
            print(f"Config file {config_name}.json not found, skipping...")
            continue
        
        print(f"Using config: {config_name}.json")

        # Run baseline on each benchmark
        for benchmark_file in sorted(benchmark_files):
            benchmark_path = Path(benchmark_file)
            total_benchmarks += 1
            
            success = run_single_baseline(
                benchmark_path, 
                config_name, 
                config_output_dir
            )
            
            if success:
                successful_benchmarks += 1

    # Summary
    print(f"\n=== Baseline Generation Complete ===")
    print(f"Total benchmarks: {total_benchmarks}")
    print(f"Successful: {successful_benchmarks}")
    print(f"Failed: {total_benchmarks - successful_benchmarks}")
    print(f"Success rate: {successful_benchmarks/total_benchmarks*100:.1f}%")
    print(f"Results saved to: {baseline_dir.absolute()}")
    
    return 0 if successful_benchmarks > 0 else 1


if __name__ == "__main__":
    sys.exit(main())