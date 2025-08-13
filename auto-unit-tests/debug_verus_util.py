import os
import sys
import verus_util as verus  # must contain your six processing functions
from dotenv import load_dotenv

load_dotenv()

# Ensure debug folder exists
DEBUG_DIR = "debug"
TEST_STRING = "/* TEST CODE BELOW */"
os.makedirs(DEBUG_DIR, exist_ok=True)

def save_stage(code, stage_num, stage_name, rust_file_path=None, last_only=False):
    """Save intermediate Rust code to debug folder."""
    if last_only and rust_file_path:
        base_name = os.path.basename(rust_file_path)
        filename = f"last_{base_name}"
    else:
        filename = f"{stage_num}_{stage_name}.rs"
    filepath = os.path.join(DEBUG_DIR, filename)
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(code)

def save_baseline(code, rust_file_path):
    """Save LLM baseline output to debug folder."""
    base_name = os.path.basename(rust_file_path)
    filename = f"base_{base_name}"
    filepath = os.path.join(DEBUG_DIR, filename)
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(code)

def main():
    if len(sys.argv) < 2:
        print("Usage: python debug.py <rust_filename> [-l] [-b]")
        sys.exit(1)

    rust_file_path = sys.argv[1]
    last_only_flag = "-l" in sys.argv
    baseline_flag = "-b" in sys.argv

    if not os.path.exists(rust_file_path):
        print(f"Error: file '{rust_file_path}' not found.")
        sys.exit(1)

    with open(rust_file_path, "r", encoding="utf-8") as f:
        rust_code = f.read()

    code = rust_code[:rust_code.find(TEST_STRING)]
    
    # If baseline flag set, run pure_llm_baseline and save result
    if baseline_flag:
        baseline_code = verus.pure_llm_baseline(code)
        save_baseline(baseline_code, rust_file_path)

    # Six stages — adjust if your actual stages differ
    stages = [
        ("remove_container", verus.remove_container),
        ("remove_specs", verus.remove_specs),
        ("remove_tracked_ghost_proof_blocks", verus.remove_tracked_ghost_proof_blocks),
        ("remove_spec_functions", verus.remove_spec_functions),
        ("final_check", verus.final_check),
        ("remove_empty", verus.remove_empty),
    ]

    for idx, (stage_name, func) in enumerate(stages, start=1):
        code = func(code)
        # Save either all stages (default) or only the last stage if -l is used
        if not last_only_flag or idx == len(stages):
            save_stage(code, idx, stage_name, rust_file_path, last_only=last_only_flag)

    print(f"Debug files saved in '{DEBUG_DIR}'.")

if __name__ == "__main__":
    main()