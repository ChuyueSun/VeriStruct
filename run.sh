#!/usr/bin/env fish

# Get the directory where the script is located
set SCRIPT_DIR (dirname (status filename))
set SCRIPT_DIR (cd $SCRIPT_DIR && pwd)

# Set PYTHONPATH to include the project root
set -x PYTHONPATH "$SCRIPT_DIR:$PYTHONPATH"

# Set project directory (default to script directory)
if not set -q VERUS_PROJECT_DIR
    set -x VERUS_PROJECT_DIR "$SCRIPT_DIR"
    echo "VERUS_PROJECT_DIR not set, using: $VERUS_PROJECT_DIR"
else
    echo "Using custom project directory: $VERUS_PROJECT_DIR"
end

# Enable LLM inference
set -x ENABLE_LLM_INFERENCE 1

# Configure LLM caching
set -x ENABLE_LLM_CACHE 1

set -x LLM_CACHE_DIR "$SCRIPT_DIR/llm_cache"
set -x LLM_CACHE_MAX_AGE_DAYS 7

echo "=== Running VerusAgent with Azure Configuration ==="
echo "Python path: $PYTHONPATH"
echo "Project directory: $VERUS_PROJECT_DIR"
echo "LLM inference enabled: $ENABLE_LLM_INFERENCE"
echo "LLM caching enabled: $ENABLE_LLM_CACHE"
echo "LLM cache directory: $LLM_CACHE_DIR"
echo "LLM cache max age: $LLM_CACHE_MAX_AGE_DAYS days"

# Change to configs directory to ensure we can find the config
cd "$SCRIPT_DIR/src/configs"

# Reset the config to use Azure
python -c "from sconfig import reset_config; reset_config('config-azure'); print('Configuration reset to config-azure')"

# Create output directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/output"
# Create tmp directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/tmp"
# Create cache directory if it doesn't exist
mkdir -p "$LLM_CACHE_DIR"

# Run the main script
cd "$SCRIPT_DIR"
python src/main.py

# Check exit status
if test $status -eq 0
    echo "✅ VerusAgent completed successfully!"
    echo "Check the '$SCRIPT_DIR/output' directory for results (using Azure LLM API with caching)"
    echo "Cache statistics: Hit rate can be checked in the logs"
else
    echo "❌ VerusAgent failed to run. Please check the error messages above."
end
