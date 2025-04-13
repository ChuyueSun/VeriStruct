import os
import sys
import json
import logging
from refinement import Refinement
from utils import AttrDict
from veval import verus
from houdini import houdini
from pathlib import Path

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

    # If you need specific immutable functions, list them here
    immutable_funcs = ['test_enqueue_dequeue_generic']  
    hdn = houdini(config, immutable_funcs)
    # Read the .rs file from the command line
    rs_path = Path(sys.argv[1])
    if not rs_path.is_file():
        print(f"Cannot find file: {rs_path}")
        sys.exit(1)
    # Optionally set up a logger
    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger(__name__)
    code = rs_path.read_text(encoding="utf-8")

    # Run houdini
    failures, new_code = hdn.run(code)

    # Report the results
    if len(failures) == 0:
        print("No verification failures remain after Houdini run!")
    else:
        print(f"Still have {len(failures)} failures:")
        for i, f in enumerate(failures, start=1):
            print(f"\nFailure {i}:")
            print(f"  Error Type: {f.error}")
            # Print out a short snippet of the failing lines:
            if f.trace:
                t = f.trace[0]
                lines_info = f"  at lines {t.lines[0]}-{t.lines[1]}"
                snippet = "\n".join(err_line.text for err_line in t.text)
                print(lines_info)
                print(f"  Snippet: {snippet}")

    # Optionally save the new code to see what changed
    # Here we just print or write to "output.rs"
    with open("output.rs", "w", encoding="utf-8") as out_f:
        out_f.write(new_code)
    print("\n[Info] Wrote the updated code to output.rs")

if __name__ == "__main__":
    main()
