#!/usr/bin/env python3
import argparse
import os
import sys
from datetime import datetime
from pathlib import Path


def display_banner(file_path=None):
    """Display a prominent banner with the input file name"""
    file_name = Path(file_path).name if file_path else "DEFAULT FILE"
    file_path_str = str(Path(file_path).absolute()) if file_path else "DEFAULT PATH"
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    output_dir = Path("output").absolute()
    banner_width = max(80, len(file_path_str) + 20)

    print("\n" + "=" * banner_width)
    print(f"{'VERISTRUCT':^{banner_width}}")
    print(f"{'PROCESSING FILE:':^{banner_width}}")
    print(f"{file_name:^{banner_width}}")
    print(f"{file_path_str:^{banner_width}}")
    print("-" * banner_width)
    print(f"{'Start Time: ' + timestamp:^{banner_width}}")
    print(f"{'Output Directory: ' + str(output_dir):^{banner_width}}")
    print("=" * banner_width + "\n")


def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(
        description="Run VeriStruct for formal verification on a single file",
        epilog="Example: python run_agent.py --test-file benchmarks-complete/vectors_todo.rs --config config-azure",
    )
    parser.add_argument(
        "--test-file",
        help="Path to the Rust file to verify (can be any .rs file)",
        default=None,
        metavar="PATH",
    )
    parser.add_argument(
        "--verus-path", help="Path to the Verus executable", default=None, metavar="PATH"
    )
    parser.add_argument(
        "--config",
        help="Config name to use, e.g., 'config-azure' (singular, one config only)",
        default="config-azure",
        metavar="NAME",
    )
    parser.add_argument(
        "--no-cache-read", action="store_true", help="Disable reading from LLM cache"
    )
    parser.add_argument(
        "--output-dir", help="Directory to store output artifacts", default="output"
    )
    parser.add_argument(
        "--immutable-functions",
        help="Comma-separated list of function names that should not be modified during generation or repair",
        default=None,
    )
    parser.add_argument("--num-repair-rounds", help="Number of repair rounds to run", default=5)
    args = parser.parse_args()

    # Set environment variables if arguments are provided
    if args.test_file:
        os.environ["VERUS_TEST_FILE"] = str(Path(args.test_file).absolute())
        print(f"Using test file: {os.environ['VERUS_TEST_FILE']}")

    if args.verus_path:
        os.environ["VERUS_PATH"] = str(Path(args.verus_path).absolute())
        print(f"Using Verus path: {os.environ['VERUS_PATH']}")

    if args.output_dir:
        target_dir = Path(args.output_dir).absolute()
        target_dir.mkdir(parents=True, exist_ok=True)

        # Expose the chosen output directory via environment variables for
        # downstream modules/helpers.
        os.environ["output_dir"] = str(target_dir)
        print(f"Using output directory: {target_dir}")

    # Set config environment variable
    os.environ["VERUS_CONFIG"] = args.config
    print(f"Using config: {args.config}")

    # Set cache read flag if specified
    if args.no_cache_read:
        os.environ["ENABLE_LLM_CACHE"] = "0"
        print("LLM cache reading disabled")

    # Set immutable functions if specified
    if args.immutable_functions:
        os.environ["VERUS_IMMUTABLE_FUNCTIONS"] = args.immutable_functions
        print(f"Using immutable functions: {args.immutable_functions}")

    # Set number of repair rounds if specified
    if args.num_repair_rounds:
        os.environ["VERUS_NUM_REPAIR_ROUNDS"] = str(args.num_repair_rounds)
        print(f"Using number of repair rounds: {args.num_repair_rounds}")

    # Set output directory env variable
    os.environ["VERUS_OUTPUT_DIR"] = str(Path(args.output_dir).absolute())
    print(f"Using output directory: {os.environ['VERUS_OUTPUT_DIR']}")

    # Display banner with file name
    display_banner(args.test_file)

    # Add the project root to Python path
    project_root = Path(__file__).parent
    sys.path.append(str(project_root))

    # Import and run the main function
    from src.main import main as verus_main

    verus_main()


if __name__ == "__main__":
    main()
