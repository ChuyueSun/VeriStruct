import json
import os
from pathlib import Path

"""
Automatically collect all json files in the configs directory and load them into a dictionary.
"""
configs = {}

# Get the directory where this script is located
current_dir = Path(__file__).parent
configs_dir = current_dir  # Since sconfig.py is already in the configs directory

try:
    for jfile in os.listdir(configs_dir):
        if jfile.endswith(".json"):
            try:
                with open(os.path.join(configs_dir, jfile), "r") as f:
                    configs[jfile[:-5]] = json.load(f)
            except json.JSONDecodeError as e:
                print(f"Error Loading Config {jfile}: {e}")
except FileNotFoundError:
    print(f"Warning: Could not find configs directory at {configs_dir}")
    # Create a default config if directory not found
    configs["config-default"] = {
        "aoai_api_base": ["https://api.openai.com/v1/"],
        "aoai_api_version": "2023-12-01-preview",
        "aoai_api_key": ["YOUR_API_KEY_HERE"],
        "aoai_max_retries": 5,
        "max_token": 8192,
        "aoai_generation_model": "gpt-4o",
        "aoai_debug_model": "gpt-4o",
        "example_path": "examples",
        "project_dir": ".",
        "tmp_dir": "tmp",
    }

# Default to config-azure instead of config-yican
if "config-azure" in configs:
    config = configs["config-azure"]
else:
    # Use the first config found or the default
    config = (
        next(iter(configs.values())) if configs else configs.get("config-default", {})
    )

# Hard code the example, lemma, and util paths
config["example_path"] = Path(__file__).parent.parent / "examples"
config["lemma_path"] = Path(__file__).parent.parent / "lemmas"
config["util_path"] = Path(__file__).parent.parent.parent / "utils"

def reset_config(config_name="config"):
    """
    Reset the global config by loading the specified config file.
    
    Args:
        config_name: Name of the config file (without extension)
    """
    global config
    
    # Determine config file path
    config_dir = Path(__file__).parent.absolute()
    config_file = config_dir / f"{config_name}.json"
    
    # Load default configuration
    with open(config_file, "r") as f:
        config = json.load(f)
    
    # Allow environment variable overrides for key settings
    # Project directory can be customized per machine via environment variable
    env_project_dir = os.environ.get("VERUS_PROJECT_DIR")
    if env_project_dir:
        config["project_dir"] = env_project_dir
        print(f"Using project directory from environment: {env_project_dir}")
    elif "project_dir" not in config:
        # Default to current directory if not specified
        config["project_dir"] = "."
        
    # Allow verus path override via environment variable as well
    env_verus_path = os.environ.get("VERUS_PATH")
    if env_verus_path:
        config["verus_path"] = env_verus_path
        print(f"Using Verus path from environment: {env_verus_path}")
    
    return config
