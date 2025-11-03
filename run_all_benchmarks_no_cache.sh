#!/bin/bash
# Run all TODO benchmarks in parallel with Azure o1, no cache read

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="benchmark_nocache_${TIMESTAMP}"
mkdir -p "$RESULTS_DIR"

echo "=========================================="
echo "Running All Benchmarks - No Cache Read"
echo "Config: Azure o1"
echo "Timestamp: ${TIMESTAMP}"
echo "Results: ${RESULTS_DIR}/"
echo "Parallelism: 8 jobs"
echo "=========================================="
echo ""

# Create list of benchmarks
cat > "$RESULTS_DIR/benchmarks.txt" << 'EOF'
atomics_todo.rs
bitmap_2_todo.rs
bitmap_todo.rs
bst_map_todo.rs
invariants_todo.rs
node_todo.rs
option_todo.rs
rb_type_invariant_todo.rs
rwlock_vstd_todo.rs
set_from_vec_todo.rs
transfer_todo.rs
treemap_todo.rs
vectors_todo.rs
EOF

echo "Total benchmarks: 13"
echo "Using xargs for parallel execution (8 concurrent jobs)"
echo ""

# Run in parallel with xargs
cat "$RESULTS_DIR/benchmarks.txt" | xargs -P 8 -I {} bash -c "
    name=\$(basename {} .rs)
    log='$RESULTS_DIR/\${name}.log'

    echo \"[\$(date +%H:%M:%S)] Starting: \$name\"

    python3 ./run_agent.py \\
        --test-file benchmarks-complete/{} \\
        --config config-azure \\
        --immutable-functions test \\
        --no-cache-read \\
        > \"\$log\" 2>&1

    exit_code=\$?

    if [ \$exit_code -eq 0 ]; then
        echo \"[\$(date +%H:%M:%S)] ✓ Completed: \$name\"
    else
        echo \"[\$(date +%H:%M:%S)] ✗ Failed: \$name (exit: \$exit_code)\"
    fi
"

echo ""
echo "=========================================="
echo "ALL BENCHMARKS COMPLETED"
echo "=========================================="
echo ""

# Generate summary
SUMMARY="$RESULTS_DIR/summary.txt"

echo "BENCHMARK RESULTS SUMMARY" | tee "$SUMMARY"
echo "Config: Azure o1" | tee -a "$SUMMARY"
echo "No Cache Read: Yes" | tee -a "$SUMMARY"
echo "Date: $(date)" | tee -a "$SUMMARY"
echo "All 8 fixes applied: Yes" | tee -a "$SUMMARY"
echo "========================================" | tee -a "$SUMMARY"
echo "" | tee -a "$SUMMARY"

for bench_file in $(cat "$RESULTS_DIR/benchmarks.txt"); do
    name="${bench_file%.rs}"
    log="$RESULTS_DIR/${name}.log"

    if [ -f "$log" ]; then
        # Find the output directory (with azure_ prefix)
        output_dir=$(ls -dt output/${name}/azure_*/ 2>/dev/null | head -1)

        if [ -n "$output_dir" ] && [ -f "$output_dir/final_result.rs" ]; then
            # Get verification result
            result=$(verus "$output_dir/final_result.rs" 2>&1 | grep "verification results" || echo "Compilation error")

            # Get time from statistics
            if [ -f "$output_dir/statistics"/report*.txt ]; then
                time=$(grep "Total Execution Time" "$output_dir/statistics"/report*.txt | awk '{print $4}' | head -1)
                success=$(grep "Verification Success" "$output_dir/statistics"/report*.txt | awk '{print $3}' | head -1)

                if [ "$success" = "Yes" ]; then
                    status="✅ $result | Time: ${time}s"
                else
                    status="⚠️  $result | Time: ${time}s"
                fi
            else
                status="⚠️  $result"
            fi
        else
            status="❌ Failed or incomplete"
        fi

        printf "%-25s %s\n" "$name" "$status" | tee -a "$SUMMARY"
    else
        printf "%-25s %s\n" "$name" "❌ LOG NOT FOUND" | tee -a "$SUMMARY"
    fi
done

echo "" | tee -a "$SUMMARY"
echo "Detailed logs: ${RESULTS_DIR}/" | tee -a "$SUMMARY"
echo "Output files: output/*/azure_<timestamp>/" | tee -a "$SUMMARY"
