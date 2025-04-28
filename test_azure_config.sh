#!/usr/bin/env fish

# Get the directory where the script is located
set SCRIPT_DIR (dirname (status filename))
set SCRIPT_DIR (cd $SCRIPT_DIR && pwd)

# Set PYTHONPATH to include the project root
set -x PYTHONPATH "$SCRIPT_DIR:$PYTHONPATH"

# Parse command line arguments
set deployment ""
set list_models false

for arg in $argv
    switch $arg
        case "--deployment=*"
            set deployment (string replace "--deployment=" "" $arg)
        case "--list-models"
            set list_models true
    end
end

# Run the test script
echo "Running Azure configuration test..."

if test -n "$deployment"
    python $SCRIPT_DIR/test_azure_config.py --deployment $deployment
else if $list_models
    python $SCRIPT_DIR/test_azure_config.py --list-models
else
    python $SCRIPT_DIR/test_azure_config.py
end

# Note about environment variables
echo ""
echo "Environment variables you can set:"
echo "1. AZURE_API_KEY - Your actual API key"
echo "2. AZURE_DEPLOYMENT_NAME - The deployment name to use"
echo ""
echo "Example for fish shell:"
echo "set -x AZURE_API_KEY your_actual_api_key"
echo "set -x AZURE_DEPLOYMENT_NAME gpt-4"
echo ""
echo "You can also use command line arguments:"
echo "./test_azure_config.sh --deployment=gpt-4"
echo "./test_azure_config.sh --list-models"
