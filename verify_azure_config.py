#!/usr/bin/env python3
"""
Simple script to verify the Azure configuration.
This just loads and displays the config-azure.json file to confirm it has the right values.
"""

import os
import sys
import json
from pathlib import Path

# Find the script directory
script_dir = Path(__file__).parent.absolute()
config_path = script_dir / "src" / "configs" / "config-azure.json"

def main():
    """Display the contents of the Azure configuration file."""
    print(f"Looking for config file at: {config_path}")
    
    if not config_path.exists():
        print(f"Error: Configuration file not found at {config_path}")
        return
        
    try:
        with open(config_path, 'r') as f:
            config = json.load(f)
            
        print("\n=== Azure Configuration ===")
        print(f"API Base: {config.get('aoai_api_base', ['Not set'])[0]}")
        print(f"API Version: {config.get('aoai_api_version', 'Not set')}")
        print(f"Generation Model: {config.get('aoai_generation_model', 'Not set')}")
        print(f"Refinement Model: {config.get('aoai_refinement_model', 'Not set')}")
        print(f"Platform: {config.get('platform', 'Not set')}")
        
        # Check API key format without revealing full key
        api_key = config.get('aoai_api_key', [''])[0]
        if api_key:
            print(f"API Key: {api_key[:4]}...{api_key[-4:]} (length: {len(api_key)})")
        else:
            print("API Key: Not specified")
            
        # Check for required fields
        required_fields = ['aoai_api_base', 'aoai_api_version', 'aoai_api_key', 
                          'aoai_generation_model', 'platform']
        missing = [field for field in required_fields if field not in config]
        
        if missing:
            print(f"\nWarning: Missing required fields: {', '.join(missing)}")
        else:
            print("\nAll required fields are present.")
            
        # Now test importing from the sconfig module
        try:
            sys.path.append(str(script_dir))
            from src.configs.sconfig import configs, reset_config, config as current_config
            
            print("\n=== Testing sconfig module ===")
            print(f"Available configs: {list(configs.keys())}")
            print(f"Current config: {current_config.get('platform', 'Not set')}")
            
            if 'config-azure' in configs:
                print("config-azure is available in sconfig.configs")
                reset_config('config-azure')
                print(f"After reset_config: {current_config.get('platform', 'Not set')}")
                print(f"API Base: {current_config.get('aoai_api_base', ['Not set'])[0]}")
            else:
                print("Error: config-azure not found in sconfig.configs")
                
        except Exception as e:
            print(f"\nError testing sconfig module: {e}")
            
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in config file: {e}")
    except Exception as e:
        print(f"Error loading config: {e}")

if __name__ == "__main__":
    main() 