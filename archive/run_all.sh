#!/bin/bash

# Set the folder where your .rs files live
FOLDER="benchmarks-complete"  # <-- change this!

for file in "$FOLDER"/*.rs
do
    echo "Running unit_test_gen.py on $file"
    python unit.py "$file"
    echo "Done with $file"
    echo "----------------------"
done