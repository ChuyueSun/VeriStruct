#!/bin/bash
# Check status of all benchmark runs

RESULTS_DIR=$(ls -dt benchmark_results_* 2>/dev/null | head -1)

if [ -z "$RESULTS_DIR" ]; then
    echo "No benchmark results directory found"
    exit 1
fi

echo "=========================================="
echo "Benchmark Status: $RESULTS_DIR"
echo "=========================================="
echo ""

# Count running processes
RUNNING=$(ps aux | grep "run_agent.py" | grep -v grep | wc -l)
echo "Active processes: $RUNNING"
echo ""

# Show progress
if [ -f "$RESULTS_DIR/progress.log" ]; then
    echo "Recent activity:"
    tail -10 "$RESULTS_DIR/progress.log"
    echo ""
fi

# Count completed
STARTED=$(grep -c "Starting:" "$RESULTS_DIR/progress.log" 2>/dev/null || echo 0)
FINISHED=$(grep -c "Finished:" "$RESULTS_DIR/progress.log" 2>/dev/null || echo 0)

echo "Progress: $FINISHED / $STARTED benchmarks completed"
echo ""

# Quick status of each
echo "Individual Status:"
echo "------------------"
for log in "$RESULTS_DIR"/*.log; do
    if [ -f "$log" ] && [ "$(basename $log)" != "progress.log" ]; then
        name=$(basename "$log" .log)
        lines=$(wc -l < "$log" 2>/dev/null || echo 0)

        if grep -q "Verification Success: Yes" "$log" 2>/dev/null; then
            status="✅ SUCCESS"
        elif grep -q "Verification Success: No" "$log" 2>/dev/null; then
            status="⚠️  PARTIAL"
        elif [ "$lines" -gt 500 ]; then
            status="🔄 RUNNING ($lines lines)"
        elif [ "$lines" -gt 50 ]; then
            status="🔄 STARTING ($lines lines)"
        else
            status="⏳ PENDING"
        fi

        printf "%-25s %s\n" "$name" "$status"
    fi
done

echo ""
echo "To watch live: watch -n 5 $0"
