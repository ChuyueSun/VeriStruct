import os
import glob
import subprocess
import argparse


def main():
    parser = argparse.ArgumentParser(description="Run all *_todo.rs benchmarks with specified configs.")
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["config-xai", "config-claude", "config-azure"],
        help="List of config file names (without .json) to pass to run_agent.py",
    )
    args = parser.parse_args()

    # Prepare results directory
    os.system("rm -rf results")
    os.makedirs("results", exist_ok=True)

    for cfg in args.configs:
        cfg_results_root = os.path.join("results", cfg)
        os.makedirs(cfg_results_root, exist_ok=True)

        for todo_path in glob.glob("benchmarks/*_todo.rs"):
            name = os.path.splitext(os.path.basename(todo_path))[0]
            test_file = f"benchmarks/{name}.rs"

            bench_dir = os.path.join(cfg_results_root, name)
            os.makedirs(bench_dir, exist_ok=True)

            log_file = os.path.join(bench_dir, "output.log")

            print(f"Running {name} with {cfg} -> log: {log_file}")

            cmd = (
                f"./run_agent.py --test-file {test_file} --config {cfg} > {log_file} 2>&1"
            )

            try:
                subprocess.run(cmd, check=True, text=True, shell=True)
            except subprocess.CalledProcessError:
                print(f"Error running {name} with {cfg}, see {log_file} for details")


if __name__ == "__main__":
    main()
    