#!/bin/bash
# Monitor no-cache benchmark runs

RESULTS_DIR=$(ls -dt benchmark_nocache_* 2>/dev/null | head -1)

if [ -z "$RESULTS_DIR" ]; then
    echo "No nocache benchmark results found"
    exit 1
fi

echo "=========================================="
echo "No-Cache Benchmark Run: $RESULTS_DIR"
echo "Config: Azure o1"
echo "=========================================="
echo ""

# Count active processes
RUNNING=$(ps aux | grep "run_agent.py" | grep -v grep | wc -l)
echo "Active processes: $RUNNING"
echo ""

# Show which benchmarks are running
if [ $RUNNING -gt 0 ]; then
    echo "Currently running:"
    ps aux | grep "run_agent.py" | grep -v grep | awk '{print $NF}' | sed 's|benchmarks-complete/||' | sort
    echo ""
fi

# Check status of each benchmark
echo "Benchmark Status:"
echo "-----------------"

for bench_file in $(cat "$RESULTS_DIR/benchmarks.txt" 2>/dev/null); do
    name="${bench_file%.rs}"
    log="$RESULTS_DIR/${name}.log"

    if [ -f "$log" ]; then
        lines=$(wc -l < "$log" 2>/dev/null || echo 0)

        if grep -q "Verification Success: Yes" "$log" 2>/dev/null; then
            status="✅ SUCCESS"
        elif grep -q "Verification Success: No" "$log" 2>/dev/null; then
            result=$(grep "verification results::" "$log" | tail -1 || echo "errors")
            status="⚠️  $result"
        elif [ "$lines" -gt 1000 ]; then
            status="🔄 RUNNING ($lines lines)"
        elif [ "$lines" -gt 100 ]; then
            status="🔄 STARTING ($lines lines)"
        else
            status="⏳ PENDING"
        fi

        printf "%-25s %s\n" "$name" "$status"
    else
        printf "%-25s %s\n" "$name" "⏳ NOT STARTED"
    fi
done

echo ""
echo "Output directories: output/*/azure_<timestamp>/"
echo "To watch live: watch -n 5 $0"
