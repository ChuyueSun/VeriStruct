#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Set PYTHONPATH to include the project root
export PYTHONPATH="$SCRIPT_DIR:$PYTHONPATH"

# Create output directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/output"

# Create tmp directory if it doesn't exist (as referenced in config)
mkdir -p "$SCRIPT_DIR/tmp"

# Run the main script from the script directory
cd "$SCRIPT_DIR"
python src/main.py

echo "Check the '$SCRIPT_DIR/output' directory for results" 