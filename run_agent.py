#!/usr/bin/env python3
import os
import sys
import argparse
from pathlib import Path

def main():
    # Parse command line arguments
    parser = argparse.ArgumentParser(description='Run VerusAgent for formal verification')
    parser.add_argument('--test-file', help='Path to the Rust file to verify', default=None)
    parser.add_argument('--verus-path', help='Path to the Verus executable', default=None)
    parser.add_argument('--config', help='Config file to use (default: config-azure)', default='config-azure')
    args = parser.parse_args()
    
    # Set environment variables if arguments are provided
    if args.test_file:
        os.environ['VERUS_TEST_FILE'] = str(Path(args.test_file).absolute())
        print(f"Using test file: {os.environ['VERUS_TEST_FILE']}")
    
    if args.verus_path:
        os.environ['VERUS_PATH'] = str(Path(args.verus_path).absolute())
        print(f"Using Verus path: {os.environ['VERUS_PATH']}")
    
    # Set config environment variable
    os.environ['VERUS_CONFIG'] = args.config
    print(f"Using config: {args.config}")
    
    # Add the project root to Python path
    project_root = Path(__file__).parent
    sys.path.append(str(project_root))
    
    # Import and run the main function
    from src.main import main as verus_main
    verus_main()

if __name__ == "__main__":
    main() 