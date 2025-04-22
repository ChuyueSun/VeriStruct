#!/usr/bin/env fish

# Get the directory where the script is located
set SCRIPT_DIR (dirname (status filename))
set SCRIPT_DIR (cd $SCRIPT_DIR && pwd)

# Source the environment variables from run.sh
# We need to extract and set them manually since we can't source directly

# Set PYTHONPATH to include the project root
set -x PYTHONPATH "$SCRIPT_DIR:$PYTHONPATH"

# Enable LLM inference
set -x ENABLE_LLM_INFERENCE 1

# Configure LLM caching
set -x LLM_CACHE_ENABLED 1
set -x LLM_CACHE_DIR "$SCRIPT_DIR/llm_cache"
set -x LLM_CACHE_MAX_AGE_DAYS 7

echo "=== Testing Caching in run.sh Environment ==="
echo "Python path: $PYTHONPATH"
echo "LLM inference enabled: $ENABLE_LLM_INFERENCE"
echo "LLM caching enabled: $LLM_CACHE_ENABLED"
echo "LLM cache directory: $LLM_CACHE_DIR"

# Create cache directory if it doesn't exist
mkdir -p "$LLM_CACHE_DIR"

# Change to configs directory to ensure we can find the config
cd "$SCRIPT_DIR/src/configs"

# Reset the config to use Azure
python -c "from sconfig import reset_config; reset_config('config-azure'); print('Configuration reset to config-azure')"

# Go back to project root
cd "$SCRIPT_DIR"

# Run the caching test
echo -e "\nRunning cache test..."
python test_run_caching.py

# Check exit status
if test $status -eq 0
    echo -e "\n✅ Cache test completed successfully!"
    
    # Display cache files
    echo -e "\nCache files in $LLM_CACHE_DIR:"
    ls -la $LLM_CACHE_DIR
    
    # Show cache stats
    set CACHE_FILES (find $LLM_CACHE_DIR -name "*.json" | wc -l)
    echo -e "\nTotal cache files: $CACHE_FILES"
    
    echo -e "\nVerification complete. The caching mechanism works properly in the run.sh environment."
else
    echo -e "\n❌ Cache test failed. Check the error messages above."
end 