#!/bin/bash
# Compare different models on node_todo benchmark

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
COMPARISON_DIR="model_comparison_${TIMESTAMP}"
mkdir -p "$COMPARISON_DIR"

echo "=========================================="
echo "Model Comparison Test on node_todo"
echo "Timestamp: ${TIMESTAMP}"
echo "Results: ${COMPARISON_DIR}/"
echo "=========================================="
echo ""

# Models to test
CONFIGS=(
    "config-azure-o1mini"
    "config-azure-o3mini"
    "config-anthropic-sonnet"
)

MODEL_NAMES=(
    "o1-mini (Azure)"
    "o3-mini (Azure)"
    "Claude Sonnet 4.5"
)

# Run each model
for i in "${!CONFIGS[@]}"; do
    config="${CONFIGS[$i]}"
    model_name="${MODEL_NAMES[$i]}"
    log_file="${COMPARISON_DIR}/$(echo $config | sed 's/config-//').log"

    echo "=========================================="
    echo "[$((i+1))/3] Testing: $model_name"
    echo "Config: $config"
    echo "Log: $log_file"
    echo "=========================================="

    python3 ./run_agent.py \
        --test-file benchmarks-complete/node_todo.rs \
        --config "$config" \
        --immutable-functions test \
        > "$log_file" 2>&1

    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "✓ Completed: $model_name"
    else
        echo "✗ Failed: $model_name (exit code: $exit_code)"
    fi

    echo ""
done

echo "=========================================="
echo "ALL MODEL TESTS COMPLETED"
echo "=========================================="
echo ""

# Generate comparison report
REPORT="${COMPARISON_DIR}/comparison_report.txt"

echo "MODEL COMPARISON REPORT" | tee "$REPORT"
echo "Benchmark: node_todo" | tee -a "$REPORT"
echo "Date: $(date)" | tee -a "$REPORT"
echo "All fixes applied: Yes (7 fixes)" | tee -a "$REPORT"
echo "========================================" | tee -a "$REPORT"
echo "" | tee -a "$REPORT"

for i in "${!CONFIGS[@]}"; do
    config="${CONFIGS[$i]}"
    model_name="${MODEL_NAMES[$i]}"
    config_short=$(echo $config | sed 's/config-//' | tr '-' '_')

    echo "=== $model_name ===" | tee -a "$REPORT"
    echo "Config: $config" | tee -a "$REPORT"

    # Find the output directory
    output_dir=$(ls -dt output/node_todo/${config_short}_*/ 2>/dev/null | head -1)

    if [ -n "$output_dir" ] && [ -f "$output_dir/final_result.rs" ]; then
        # Get verification result
        result=$(verus "$output_dir/final_result.rs" 2>&1 | grep "verification results" || echo "Verification error")
        echo "Result: $result" | tee -a "$REPORT"

        # Get statistics
        if [ -f "$output_dir/statistics"/report*.txt ]; then
            exec_time=$(grep "Total Execution Time" "$output_dir/statistics"/report*.txt | head -1)
            llm_calls=$(grep "Total LLM Calls" "$output_dir/statistics"/report*.txt | head -1)
            cache_hit=$(grep "Cache Hit Rate" "$output_dir/statistics"/report*.txt | head -1)

            echo "$exec_time" | tee -a "$REPORT"
            echo "$llm_calls" | tee -a "$REPORT"
            echo "$cache_hit" | tee -a "$REPORT"
        fi

        echo "Output: $output_dir" | tee -a "$REPORT"
    else
        echo "Status: Run failed or output not found" | tee -a "$REPORT"
    fi

    echo "" | tee -a "$REPORT"
done

echo "========================================" | tee -a "$REPORT"
echo "Detailed logs in: ${COMPARISON_DIR}/" | tee -a "$REPORT"
echo "Comparison report: ${REPORT}" | tee -a "$REPORT"
echo ""
echo "View report: cat ${REPORT}"
