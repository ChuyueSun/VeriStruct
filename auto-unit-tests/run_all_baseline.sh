#!/bin/bash

# Make sure the script fails on any error
set -e

# Change this if your folder is named differently
ROOT_DIR="benchmarks-complete"

# Loop through each .rs file under the root directory
find "$ROOT_DIR" -name "*.rs" | while read -r file; do
    echo "Running unit_test_gen.py on $file with baseline"
    python unit_test_gen.py "$file" -b
done