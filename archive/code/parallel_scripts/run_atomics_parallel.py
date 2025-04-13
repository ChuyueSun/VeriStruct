import subprocess
import os
from multiprocessing import Pool
from typing import List, Dict, Any

# Configuration parameters
INPUT_FILE = "../verus_lang_benchmarks/atomics_todo.rs"
OUTPUT_DIR = "o1/atomics"
CONFIG_FILE = "config-azure.json"
IMMUTABLE_FUNCTION = "main"
NUM_PROCESSES = 5
OUTPUT_RANGE = (1, 6)  # Inclusive start, exclusive end

def generate_commands(
    input_file: str, 
    output_dir: str, 
    config_file: str, 
    immutable_function: str,
    output_range: tuple
) -> List[str]:
    """Generate commands with parameterized values."""
    os.makedirs(output_dir, exist_ok=True)
    
    commands = []
    for i in range(*output_range):
        output_file = f"{output_dir}/atomics_{i}.rs"
        cmd = (
            f"python main.py "
            f"--input {input_file} "
            f"--output {output_file} "
            f"--config {config_file} "
            f"--immutable-functions {immutable_function}"
        )
        commands.append(cmd)
    
    return commands

def run_command(cmd: str) -> subprocess.CompletedProcess:
    """Execute a shell command and handle possible errors."""
    try:
        print(f"Running: {cmd}")
        result = subprocess.run(cmd, shell=True, check=True, text=True, 
                               capture_output=True)
        print(f"Command completed successfully")
        return result
    except subprocess.CalledProcessError as e:
        print(f"Command failed with error: {e}")
        print(f"Error output: {e.stderr}")
        return e

if __name__ == "__main__":
    # Generate commands using configuration parameters
    commands = generate_commands(
        input_file=INPUT_FILE,
        output_dir=OUTPUT_DIR,
        config_file=CONFIG_FILE,
        immutable_function=IMMUTABLE_FUNCTION,
        output_range=OUTPUT_RANGE
    )
    
    # Execute commands in parallel
    with Pool(processes=NUM_PROCESSES) as pool:
        results = pool.map(run_command, commands)
    
    # Summary
    success_count = sum(1 for r in results if isinstance(r, subprocess.CompletedProcess) and r.returncode == 0)
    print(f"Completed: {success_count}/{len(commands)} commands successful")
