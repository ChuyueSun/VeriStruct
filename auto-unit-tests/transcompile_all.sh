#!/bin/bash

# Make sure the script fails on any error
set -e

# Change this if your folder is named differently
ROOT_DIR="test_cases"

# Loop through each .rs file under the root directory
find "$ROOT_DIR" -name "*.rs" | while read -r file; do
    echo "Running transcompiler.py on $file"
    python transcompiler.py "$file"
done