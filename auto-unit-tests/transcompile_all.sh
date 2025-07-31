#!/bin/bash

set -e

RUST_DIR="test-cases"
SPEC_DIR="verus-test-cases"

find "$RUST_DIR" -name "*.rs" | while read -r rust_file; do
    base_name=$(basename "$rust_file")
    stem="${base_name%_tests.rs}"
    spec_file="$SPEC_DIR/${stem}_spec.rs"

    echo "→ Running: python transcompiler.py \"$rust_file\" \"$spec_file\""

    if [ ! -f "$spec_file" ]; then
        echo "Warning: Spec file not found for $base_name, skipping."
        continue
    fi

    python transcompiler.py "$rust_file" "$spec_file"
done