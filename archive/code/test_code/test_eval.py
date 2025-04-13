#!/usr/bin/env python3
"""
Test Script for Debugging get_one_failure() with a Real File

This script:
  - Loads the configuration from /home/chuyue/-verusyth/code/config-o3mini.json.
  - Sets the Verus binary path from the config.
  - Reads the file '/home/chuyue/-verusyth/code/intermediate-rb_type_inv_debug/repair/repair-2-0-Other.rs'.
  - Instantiates a VEval object and runs eval() (with JSON mode enabled).
  - Prints out the raw outputs, parsed errors, and then uses Refinement.get_one_failure()
    to select and print the prioritized error.
"""

import os
import sys
import json
import logging
from veval import VEval, verus
from refinement import Refinement
from utils import AttrDict

def main():
    # Set up logging
    logging.basicConfig(level=logging.DEBUG)
    logger = logging.getLogger("TestGetOneFailureReal")

    # Load configuration from the specified config file
    config_path = '/home/chuyue/-verusyth/code/config-o3mini.json'
    if not os.path.isfile(config_path):
        logger.error(f"Config file not found: {config_path}")
        sys.exit(1)
    with open(config_path, 'r') as cf:
        config_data = json.load(cf)
    config = AttrDict(config_data)

    # Set the verus path using the config
    if not config.verus_path:
        logger.error("verus_path not found in config.")
        sys.exit(1)
    verus.set_verus_path(config.verus_path)
    logger.info(f"Verus path set to: {verus.verus_path}")

    # The file to test
    file_path = '/home/chuyue/-verusyth/code/intermediate-rb_type_inv_8/view-0-requires_inference-4.rs'
    if not os.path.isfile(file_path):
        logger.error(f"Test file not found: {file_path}")
        sys.exit(1)

    # Read the file contents
    with open(file_path, "r") as f:
        code = f.read()

    # Instantiate VEval and run evaluation
    v = VEval(code, logger=logger)
    print("=== Running VEval ===")
    print(v.verus_succeed())
    v.eval(max_errs=5, json_mode=True)
    print(v.verus_succeed())

    print("====score=====", v.get_score())
    
    # Print diagnostic outputs
    print("=== Verus Out (stdout) ===")
    print(v.verus_out)
    print("\n=== Rustc Out (stderr) ===")
    print(v.rustc_out)
    print("\n=== Verus Result (parsed JSON) ===")
    print(v.verus_result)
    
    # Print each verus error captured
    print("\n=== Captured Verus Errors ===")
    for idx, err in enumerate(v.verus_errors):
        print(f"Error {idx}: {err}")
        print("Error Text:", err.get_text())
        print("-" * 40)
    
    # Get failures via get_failures()
    failures = v.get_failures()
    print("\n=== get_failures() Output ===")
    if not failures:
        print("No failures returned!")
    else:
        for idx, failure in enumerate(failures):
            print(f"Failure {idx}: {failure}")
            print("Failure Text:", failure.get_text())
            print("-" * 40)

    # Instantiate a Refinement object using the loaded config
    refinement = Refinement(config=config, logger=logger, immutable_funcs=[])
    
    try:
        selected_failure = refinement.get_one_failure(failures)
        logger.info("Selected failure:")
        logger.info(f"{selected_failure}")
        logger.info(f"Failure Text:\n{selected_failure.get_text()}")
    except Exception as e:
        logger.error(f"Error in get_one_failure: {e}")

if __name__ == "__main__":
    main()
