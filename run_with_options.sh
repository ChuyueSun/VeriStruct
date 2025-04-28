#!/usr/bin/env fish

# VerusAgent run script with command-line options
# Usage: ./run_with_options.sh [options]

# Default values
set USE_LLM 1
set USE_CACHE 1
set CONFIG_TYPE "config-azure"
set TEST_FILE "tests/rb_type_invariant_todo.rs"
set CACHE_MAX_AGE 7
set CLEAR_CACHE 0

# Get the directory where the script is located
set SCRIPT_DIR (dirname (status filename))
set SCRIPT_DIR (cd $SCRIPT_DIR && pwd)

function print_help
    echo "Usage: ./run_with_options.sh [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                 Show this help message"
    echo "  -l, --llm [0|1]            Enable or disable LLM inference (default: 1)"
    echo "  -c, --cache [0|1]          Enable or disable LLM caching (default: 1)"
    echo "  -t, --test-file FILE       Specify test file to verify (default: tests/rb_type_invariant_todo.rs)"
    echo "  -f, --config TYPE          Specify config type to use (default: config-azure)"
    echo "  -a, --cache-age DAYS       Set cache max age in days (default: 7)"
    echo "  -x, --clear-cache          Clear the cache before running"
    echo ""
    echo "Examples:"
    echo "  ./run_with_options.sh --llm 1 --cache 0     # Run with LLM enabled, cache disabled"
    echo "  ./run_with_options.sh -t tests/custom.rs    # Run with a custom test file"
    echo "  ./run_with_options.sh -t benchmarks/option_todo.rs -x   # Run benchmark with cleared cache"
    exit 0
end

# Parse command line arguments
set i 1
while test $i -le (count $argv)
    set arg $argv[$i]
    switch $arg
        case -h --help
            print_help

        case -l --llm
            if test $i -lt (count $argv)
                set i (math $i + 1)
                set USE_LLM $argv[$i]
                if test $USE_LLM -ne 0 -a $USE_LLM -ne 1
                    echo "Error: LLM option must be 0 or 1"
                    exit 1
                end
            end

        case -c --cache
            if test $i -lt (count $argv)
                set i (math $i + 1)
                set USE_CACHE $argv[$i]
                if test $USE_CACHE -ne 0 -a $USE_CACHE -ne 1
                    echo "Error: Cache option must be 0 or 1"
                    exit 1
                end
            end

        case -t --test-file
            if test $i -lt (count $argv)
                set i (math $i + 1)
                set TEST_FILE $argv[$i]
                if not test -f "$SCRIPT_DIR/$TEST_FILE"
                    echo "Error: Test file not found: $TEST_FILE"
                    exit 1
                end
            end

        case -f --config
            if test $i -lt (count $argv)
                set i (math $i + 1)
                set CONFIG_TYPE $argv[$i]
            end

        case -a --cache-age
            if test $i -lt (count $argv)
                set i (math $i + 1)
                set CACHE_MAX_AGE $argv[$i]
                if not string match -qr '^[0-9]+$' $CACHE_MAX_AGE
                    echo "Error: Cache age must be a positive integer"
                    exit 1
                end
            end

        case -x --clear-cache
            set CLEAR_CACHE 1

        case '*'
            echo "Unknown option: $arg"
            echo "Use --help to see available options"
            exit 1
    end
    set i (math $i + 1)
end

# Set environment variables for Python script
set -x PYTHONPATH "$SCRIPT_DIR:$PYTHONPATH"

# Configure LLM settings
set -x ENABLE_LLM_INFERENCE $USE_LLM
set -x ENABLE_LLM_CACHE $USE_CACHE
set -x LLM_CACHE_DIR "$SCRIPT_DIR/llm_cache"
set -x LLM_CACHE_MAX_AGE_DAYS $CACHE_MAX_AGE

# Set custom test file env var (will be used in main.py if present)
set -x VERUS_TEST_FILE "$TEST_FILE"

echo "=== Running VerusAgent with the following configuration ==="
echo "Python path: $PYTHONPATH"
echo "LLM inference enabled: $ENABLE_LLM_INFERENCE"
echo "LLM caching enabled: $ENABLE_LLM_CACHE"
echo "LLM cache directory: $LLM_CACHE_DIR"
echo "LLM cache max age: $LLM_CACHE_MAX_AGE_DAYS days"
echo "Test file: $VERUS_TEST_FILE"
echo "Config type: $CONFIG_TYPE"

# Change to configs directory to ensure we can find the config
cd "$SCRIPT_DIR/src/configs"

# Reset the config to the specified type
python -c "from sconfig import reset_config; reset_config('$CONFIG_TYPE'); print('Configuration reset to $CONFIG_TYPE')"

# Create output directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/output"
# Create tmp directory if it doesn't exist
mkdir -p "$SCRIPT_DIR/tmp"
# Create cache directory if it doesn't exist
mkdir -p "$LLM_CACHE_DIR"

# Clear cache if requested
if test $CLEAR_CACHE -eq 1
    echo "Clearing LLM cache directory..."
    rm -rf "$LLM_CACHE_DIR"/*
    mkdir -p "$LLM_CACHE_DIR"
end

# Run the main script
cd "$SCRIPT_DIR"
python src/main.py

# Check exit status
if test $status -eq 0
    echo "✅ VerusAgent completed successfully!"
    echo "Check the '$SCRIPT_DIR/output' directory for results"
    echo "Progress logs can be found in: '$SCRIPT_DIR/output/progress_logs/'"
else
    echo "❌ VerusAgent failed to run. Please check the error messages above."
end
