#!/usr/bin/env python3
"""
Test Script for repair_veval() in Refinement

This script performs the following steps:
  1. Loads the configuration from '/home/chuyue/-verusyth/code/config-o3mini.json'.
  2. Sets the Verus binary path using the configuration.
  3. Instantiates a Refinement object using the loaded configuration and a logger.
  4. Reads a Rust source file from 
     '/home/chuyue/-verusyth/code/intermediate-rb_type_inv_debug/repair/repair-2-0-Other.rs'.
  5. Calls repair_veval() on the Rust code.
  6. Prints the final repaired Verus code.
  
Make sure that the configuration file, the Rust file, and the Verus binary exist at the specified paths.
"""

import os
import sys
import json
import logging
from refinement import Refinement
from utils import AttrDict
from veval import verus

def main():
    # Set up logging
    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger("TestRepairVeval")

    print("=== Starting repair_veval test ===")
    # Load configuration from config-o3mini.json
    config_path = '/home/chuyue/-verusyth/code/config-4o.json'
    if not os.path.isfile(config_path):
        logger.error("Config file not found: %s", config_path)
        sys.exit(1)
    with open(config_path, 'r') as f:
        config_data = json.load(f)
    config = AttrDict(config_data)

    # Set the Verus binary path from the config
    if not hasattr(config, "verus_path") or not config.verus_path:
        logger.error("Config does not contain a valid 'verus_path'.")
        sys.exit(1)
    verus.set_verus_path(config.verus_path)
    logger.info("Verus path set to: %s", verus.verus_path)

    # Read the input Rust file
    rs_file = '/home/chuyue/-verusyth/code/intermediate-rb_type_inv_debug/repair/repair-5-0-PostCondFail.rs'
    rs_file = '/home/chuyue/-verusyth/code/intermediate-rb_type_inv_debug/view-best.rs'
    rs_file = '/home/chuyue/-verusyth/code/intermediate-rb_type_complete_1/repair/repair-5-0-MismatchedType.rs'
    rs_file = '/home/chuyue/-verusyth/code/intermediate-rb_type_complete_1/view-1-requires_inference-0.rs'
    # rs_file = '/home/chuyue/-verusyth/rb_type_invariant.rs'
    rs_file = '/home/chuyue/-verusyth/code/intermediate-rb_planner_5/view-best.rs'
    print("=== Reading Rust file ===")
    print(rs_file)
    if not os.path.isfile(rs_file):
        logger.error("Rust file not found: %s", rs_file)
        sys.exit(1)
    with open(rs_file, 'r') as f:
        code = f.read()

    logger.info("Starting repair_veval test on file: %s", rs_file)
    
    # Instantiate a Refinement object with the loaded configuration and logger
    refinement = Refinement(config=config, logger=logger, immutable_funcs=["test_enqueue_dequeue_generic"])
    print("=== Refinement object instantiated ===")
    # Call repair_veval; adjust max_attempt, func_name, temp_dir, and temp as needed.
    try:
        repaired_code = refinement.repair_veval(code, max_attempt=5, func_name=None, temp_dir="tmp", temp=1.0)
        logger.info("Repaired code obtained successfully.")
    except Exception as e:
        logger.error("Error during repair_veval: %s", e)
        sys.exit(1)

    # Print the final repaired code
    print("=== Final Repaired Code ===")
    # print(repaired_code)

    # Save the repaired code to a file
    repaired_file = os.path.join(os.path.dirname(rs_file), "repaired.rs")
    with open(repaired_file, 'w') as f:
        f.write(repaired_code)
    logger.info("Repaired code saved to: %s", repaired_file)
    print("=== Repaired code saved ===")
if __name__ == "__main__":
    main()
