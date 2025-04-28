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

# Default Set to our verusagent config or fallback to any config
if "config-yican" in configs:
    config = configs["config-yican"]
else:
    # Use the first config found or the default
    config = (
        next(iter(configs.values())) if configs else configs.get("config-default", {})
    )

# Hard code the example, lemma, and util paths
config["example_path"] = Path(__file__).parent.parent / "examples"
config["lemma_path"] = Path(__file__).parent.parent / "lemmas"
config["util_path"] = Path(__file__).parent.parent.parent / "utils"


def reset_config(name: str):
    """
    Reset the config to the specified name.
    :param name: The name of the config to reset to.
    """
    global config
    if name in configs:
        config = configs[name]
    else:
        print(f"Config {name} not found. Available configs: {list(configs.keys())}")
        print("Using current config instead.")
