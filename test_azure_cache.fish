#!/usr/bin/env fish

# Get the script directory
set SCRIPT_DIR (dirname (status filename))
set SCRIPT_DIR (cd $SCRIPT_DIR && pwd)

# Set required environment variables
set -x PYTHONPATH "$SCRIPT_DIR:$PYTHONPATH"
set -x ENABLE_LLM_INFERENCE 1
set -x LLM_CACHE_ENABLED 1
set -x LLM_CACHE_DIR "$SCRIPT_DIR/llm_cache"
set -x LLM_CACHE_MAX_AGE_DAYS 7

# Process arguments
set USE_FIXED_QUERY false
if test (count $argv) -gt 0
    if test "$argv[1]" = "--fixed-query"
        set USE_FIXED_QUERY true
        echo "⚠️ Using fixed query mode for testing cache hits"
    end
end

echo "=== Testing LLM Cache with Azure Configuration ==="
echo "Cache directory: $LLM_CACHE_DIR"
echo "LLM Cache enabled: $LLM_CACHE_ENABLED"
echo "Max cache age: $LLM_CACHE_MAX_AGE_DAYS days"

# Create cache directory if it doesn't exist
mkdir -p "$LLM_CACHE_DIR"

# Change to the project root directory
cd "$SCRIPT_DIR"

# Run the script that uses the actual Azure configuration
echo -e "\nRunning cache test with Azure config..."

if $USE_FIXED_QUERY
    python test_cache_with_actual_config.py --fixed-query
else
    python test_cache_with_actual_config.py
end

# Check the exit status
if test $status -eq 0
    echo -e "\n✅ Azure cache test completed successfully!"
    
    # Display cache files for review
    echo -e "\nCache files in $LLM_CACHE_DIR:"
    ls -la $LLM_CACHE_DIR
    
    # Count and show cache hit rate
    set CACHE_FILES (find $LLM_CACHE_DIR -name "*.json" | wc -l)
    echo -e "\nTotal cache files: $CACHE_FILES"
    
    if not $USE_FIXED_QUERY
        echo -e "\nTip: Run this test again with --fixed-query to see a cache hit on the first inference call:"
        echo "  fish test_azure_cache.fish --fixed-query"
    else
        echo -e "\nSuccessfully demonstrated the caching feature using a fixed query."
    end
else
    echo -e "\n❌ Azure cache test failed. Check the error messages above."
end 