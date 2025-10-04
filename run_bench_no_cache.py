#!/usr/bin/env python3
"""
Run benchmarks with cache disabled for accurate runtime statistics.
"""
import argparse
import glob
import os
import subprocess


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
    args = parser.parse_args()

    # Prepare results directory
    os.system("rm -rf results")
    os.makedirs("results", exist_ok=True)

    for cfg in args.configs:
        cfg_results_root = os.path.join("results", cfg)
        os.makedirs(cfg_results_root, exist_ok=True)

        for todo_path in glob.glob("benchmarks-complete/*_todo.rs"):
            name = os.path.splitext(os.path.basename(todo_path))[0]
            test_file = f"benchmarks-complete/{name}.rs"

            bench_dir = os.path.join(cfg_results_root, name)
            os.makedirs(bench_dir, exist_ok=True)

            log_file = os.path.join(bench_dir, "output.log")

            print(f"Running {name} with {cfg} (cache disabled) -> log: {log_file}")

            # Set environment to disable cache
            env = os.environ.copy()
            env["ENABLE_LLM_CACHE"] = "0"

            cmd = [
                "./run_agent.py",
                "--test-file",
                test_file,
                "--immutable-functions",
                "test",
            ]

            try:
                with open(log_file, "w") as log:
                    subprocess.run(
                        cmd,
                        env=env,
                        stdout=log,
                        stderr=subprocess.STDOUT,
                        check=True,
                        text=True,
                    )
                print(f"  ✓ Completed {name}")
            except subprocess.CalledProcessError:
                print(f"  ✗ Error running {name}, see {log_file} for details")


if __name__ == "__main__":
    main()
