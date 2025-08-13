#!/bin/bash

# Exit on any error
set -e

# Change this if your folder is named differently
ROOT_DIR="benchmarks-complete"

# Loop through each .rs file under the root directory
find "$ROOT_DIR" -name "*.rs" | while read -r file; do
    echo "Running debug_verus_util.py on $file with -l and -b flags"
    python debug_verus_util.py "$file" -l -b
done