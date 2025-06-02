#!/usr/bin/env python3
import os
import sys
import argparse
from pathlib import Path
from datetime import datetime

def display_banner(file_path=None):
    """Display a prominent banner with the input file name"""
    file_name = Path(file_path).name if file_path else "DEFAULT FILE"
    file_path_str = str(Path(file_path).absolute()) if file_path else "DEFAULT PATH"
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    output_dir = Path("output").absolute()
    banner_width = max(80, len(file_path_str) + 20)
    
    print("\n" + "=" * banner_width)
    print(f"{'VERUS AGENT':^{banner_width}}")
    print(f"{'PROCESSING FILE:':^{banner_width}}")
    print(f"{file_name:^{banner_width}}")
    print(f"{file_path_str:^{banner_width}}")
    print("-" * banner_width)
    print(f"{'Start Time: ' + timestamp:^{banner_width}}")
    print(f"{'Output Directory: ' + str(output_dir):^{banner_width}}")
    print("=" * banner_width + "\n")

def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(description='Run VerusAgent for formal verification')
    parser.add_argument('--test-file', help='Path to the Rust file to verify', default=None)
    parser.add_argument('--verus-path', help='Path to the Verus executable', default=None)
    parser.add_argument('--config', help='Config file to use (default: config-azure)', default='config-azure')
    parser.add_argument('--no-cache-read', action='store_true', help='Disable reading from LLM cache')
    parser.add_argument('--output-dir', help='Directory to store output files', default='output')
    args = parser.parse_args()
    
    # Set environment variables if arguments are provided
    if args.test_file:
        os.environ['VERUS_TEST_FILE'] = str(Path(args.test_file).absolute())
        print(f"Using test file: {os.environ['VERUS_TEST_FILE']}")
    
    if args.verus_path:
        os.environ['VERUS_PATH'] = str(Path(args.verus_path).absolute())
        print(f"Using Verus path: {os.environ['VERUS_PATH']}")

    if args.output_dir:
        os.environ['output_dir'] = str(Path(args.output_dir).absolute())
        os.system('rm -rf ' + os.environ['output_dir'])
        os.makedirs(os.environ['output_dir'], exist_ok=True)
        print(f"Using output directory: {os.environ['output_dir']}")
    
    # Set config environment variable
    os.environ['VERUS_CONFIG'] = args.config
    print(f"Using config: {args.config}")
    
    # Set cache read flag if specified
    if args.no_cache_read:
        os.environ['ENABLE_LLM_CACHE'] = '0'
        print("LLM cache reading disabled")
    
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
