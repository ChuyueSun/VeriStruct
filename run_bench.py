import argparse
import glob
import os
import subprocess


def main():
    parser = argparse.ArgumentParser(
        description="Run benchmarks from benchmarks-complete/ directory with one or more configs",
        epilog="""Examples:
  Single benchmark, single config:
    python run_bench.py --configs config-azure --benchmark vectors_todo

  Single benchmark, multiple configs (for comparison):
    python run_bench.py --configs config-azure config-openai --benchmark vectors_todo

  All benchmarks:
    python run_bench.py --configs config-azure
""",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["config-azure"],
        help="One or more config names (without .json), e.g., 'config-azure config-openai'",
        metavar="NAME",
    )
    parser.add_argument(
        "--benchmark",
        help="Benchmark name only (e.g., 'vectors_todo', NOT full path). Omit to run all benchmarks.",
        metavar="NAME",
    )
    args = parser.parse_args()

    # Prepare results directory
    os.system("rm -rf results")
    os.makedirs("results", exist_ok=True)

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
        cfg_results_root = os.path.join("results", cfg)
        os.makedirs(cfg_results_root, exist_ok=True)

        for benchmark_name in benchmarks:
            test_file = f"benchmarks-complete/{benchmark_name}.rs"

            bench_dir = os.path.join(cfg_results_root, benchmark_name)
            os.makedirs(bench_dir, exist_ok=True)

            log_file = os.path.join(bench_dir, "output.log")

            print(f"Running {benchmark_name} with {cfg} -> log: {log_file}")

            cmd = f"./run_agent.py --test-file {test_file} --immutable-functions test > {log_file} 2>&1"

            try:
                subprocess.run(cmd, check=True, text=True, shell=True)
            except subprocess.CalledProcessError:
                print(f"Error running {benchmark_name} with {cfg}, see {log_file} for details")


if __name__ == "__main__":
    main()
