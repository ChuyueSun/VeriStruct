#!/usr/bin/env fish
# VerusAgent Customization Settings for your environment
# Run with: source customize.fish && ./run.sh

# Project directory - set to your VerusAgent root
set -x VERUS_PROJECT_DIR "/home/chuyue/VerusAgent"

# Verus executable path - set to your actual Verus binary
set -x VERUS_PATH "/home/chuyue/verus/source/target-verus/release/verus"

# Optional: Set a custom test file
# Uncomment and modify this line to use a specific test file
# set -x VERUS_TEST_FILE "/home/chuyue/VerusAgent/tests/rb_type_invariant_todo.rs"

# Keep LLM inference enabled
set -x ENABLE_LLM_INFERENCE 1
set -x ENABLE_LLM_CACHE 1

echo "Custom environment variables set!"
echo "VERUS_PROJECT_DIR: $VERUS_PROJECT_DIR"
echo "VERUS_PATH: $VERUS_PATH"
if set -q VERUS_TEST_FILE
    echo "VERUS_TEST_FILE: $VERUS_TEST_FILE"
end 