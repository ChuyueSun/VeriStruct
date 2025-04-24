#!/usr/bin/env python3
"""
Test script to validate Azure configuration.
This script loads the config-azure.json file and attempts to make a simple API call
to verify the connection and credentials.
"""

import sys
import os
import json
import argparse
from pathlib import Path

# Add the project root to Python path
script_dir = Path(__file__).parent.absolute()
sys.path.append(str(script_dir))

try:
    import requests
    from dotenv import load_dotenv
except ImportError:
    print("Installing required packages...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests", "python-dotenv"])
    import requests
    from dotenv import load_dotenv

# Load environment variables from .env file if it exists
load_dotenv()

def load_config():
    """Load the Azure configuration file."""
    config_path = script_dir / "src" / "configs" / "config-azure.json"
    if not config_path.exists():
        print(f"Error: Configuration file not found at {config_path}")
        sys.exit(1)
    
    try:
        with open(config_path, 'r') as f:
            config = json.load(f)
        return config
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in config file: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error loading config: {e}")
        sys.exit(1)

def test_azure_connection(config, deployment_name=None):
    """Test the connection to Azure using the provided configuration."""
    print("Testing Azure API connection...")
    
    # Extract configuration values
    try:
        api_base = config["aoai_api_base"][0]
        api_key = config["aoai_api_key"][0]
        api_version = config["aoai_api_version"]
        model = deployment_name or config["aoai_generation_model"]
    except KeyError as e:
        print(f"Error: Missing required configuration key: {e}")
        return False
    
    # Replace placeholder API key if specified in env vars
    if api_key == "YOUR_API_KEY_HERE" or "AAABBB" in api_key:
        api_key = os.environ.get("AZURE_API_KEY", "")
        if not api_key:
            print("Error: API key is a placeholder and no AZURE_API_KEY environment variable is set")
            return False
    
    print(f"Using deployment name: {model}")
    endpoint = f"{api_base}openai/deployments/{model}/chat/completions?api-version={api_version}"
    
    headers = {
        "Content-Type": "application/json",
        "api-key": api_key
    }
    
    data = {
        "messages": [
            {"role": "system", "content": "You are a helpful AI assistant."},
            {"role": "user", "content": "Hello, this is a test of the Azure OpenAI connection."}
        ],
        "max_tokens": 50
    }
    
    try:
        print(f"Sending request to: {endpoint}")
        response = requests.post(endpoint, headers=headers, json=data, timeout=10)
        
        if response.status_code == 200:
            print("Success! Connection to Azure OpenAI API established.")
            print("\nResponse preview:")
            try:
                content = response.json()["choices"][0]["message"]["content"]
                print(f"AI response: {content[:100]}...")
            except (KeyError, IndexError) as e:
                print(f"Unexpected response format: {e}")
                print(response.json())
            return True
        else:
            print(f"Error: API request failed with status code {response.status_code}")
            print(f"Response: {response.text}")
            
            # For deploymentNotFound error, provide suggestions
            if response.status_code == 404 and "DeploymentNotFound" in response.text:
                print("\n💡 Suggestion: The specified deployment doesn't exist.")
                print("1. Check available deployments in your Azure OpenAI Studio")
                print("2. Try using 'gpt-4' or 'gpt-35-turbo' as these are common deployment names")
                print("3. Use the --deployment flag to specify a different deployment name")
            
            return False
    except requests.exceptions.RequestException as e:
        print(f"Error: Failed to connect to Azure API: {e}")
        return False

def list_common_deployment_models():
    """List common deployment model names for Azure OpenAI."""
    print("\nCommon Azure OpenAI deployment names:")
    print("- gpt-4")
    print("- gpt-4-32k")
    print("- gpt-35-turbo (GPT-3.5 Turbo)")
    print("- text-davinci-003")
    print("- text-embedding-ada-002 (for embeddings)")
    print("\nYou can try different models using: --deployment MODEL_NAME")

def main():
    """Main function to test the Azure configuration."""
    parser = argparse.ArgumentParser(description="Test Azure OpenAI API configuration")
    parser.add_argument("--deployment", type=str, help="Specify Azure deployment name")
    parser.add_argument("--list-models", action="store_true", help="List common deployment model names")
    args = parser.parse_args()
    
    if args.list_models:
        list_common_deployment_models()
        return
    
    # Get deployment name from argument or environment variable
    deployment_name = args.deployment or os.environ.get("AZURE_DEPLOYMENT_NAME")
    
    print("=== Azure Configuration Test ===")
    
    # Load configuration
    config = load_config()
    print("\nConfiguration loaded:")
    print(f"API Base: {config.get('aoai_api_base', ['Not specified'])[0]}")
    print(f"API Version: {config.get('aoai_api_version', 'Not specified')}")
    if deployment_name:
        print(f"Model: {deployment_name} (override)")
    else:
        print(f"Model: {config.get('aoai_generation_model', 'Not specified')}")
    print(f"Platform: {config.get('platform', 'Not specified')}")
    
    # Test API key format without revealing it
    api_key = config.get('aoai_api_key', [''])[0]
    if api_key:
        print(f"API Key: {api_key[:4]}...{api_key[-4:]} (length: {len(api_key)})")
    else:
        print("API Key: Not specified")
    
    print("\n=== Testing Connection ===")
    success = test_azure_connection(config, deployment_name)
    
    if success:
        print("\n✅ Configuration test passed! Your Azure setup appears to be working correctly.")
    else:
        print("\n❌ Configuration test failed. Please check the error messages above.")
        list_common_deployment_models()

if __name__ == "__main__":
    main() 