#!/bin/bash
# Script to check benchmark completion status

echo "======================================"
echo "Benchmark Status Check"
echo "======================================"
echo ""

# Check if benchmark process is running
echo "1. Process Status:"
if ps aux | grep -q "[p]ython.*bench_no_cache"; then
    echo "   ✓ Benchmarks are RUNNING"
    RUNNING=1
else
    echo "   ✗ Benchmarks have COMPLETED or stopped"
    RUNNING=0
fi
echo ""

# Count total benchmarks
TOTAL=$(ls benchmarks-complete/*_todo.rs 2>/dev/null | wc -l)
echo "2. Total Benchmarks: $TOTAL"
echo ""

# Count completed benchmarks
COMPLETED=$(find results/config-azure -name "output.log" -type f 2>/dev/null | wc -l)
echo "3. Completed Benchmarks: $COMPLETED / $TOTAL"
echo ""

# Show which benchmarks completed
echo "4. Completed Benchmark List:"
if [ -d "results/config-azure" ]; then
    ls -1 results/config-azure/ 2>/dev/null | while read bench; do
        if [ -f "results/config-azure/$bench/output.log" ]; then
            # Check if it has statistics
            if [ -d "results/config-azure/$bench/statistics" ] || [ -d "output/statistics" ]; then
                echo "   ✓ $bench (with statistics)"
            else
                echo "   ✓ $bench"
            fi
        fi
    done
else
    echo "   (No results directory yet)"
fi
echo ""

# Check main log file
echo "5. Main Log Status:"
if [ -f "benchmark_nocache_run.log" ]; then
    LINES=$(wc -l < benchmark_nocache_run.log)
    echo "   Log file: benchmark_nocache_run.log ($LINES lines)"
    echo "   Last 3 lines:"
    tail -3 benchmark_nocache_run.log | sed 's/^/      /'
else
    echo "   (No main log file found)"
fi
echo ""

# Overall status
echo "6. Overall Status:"
if [ $RUNNING -eq 1 ]; then
    PERCENT=$((COMPLETED * 100 / TOTAL))
    echo "   🔄 IN PROGRESS: $COMPLETED/$TOTAL complete ($PERCENT%)"
    echo ""
    echo "   Check again with: ./check_benchmark_status.sh"
    echo "   Or watch progress: tail -f benchmark_nocache_run.log"
elif [ $COMPLETED -eq $TOTAL ]; then
    echo "   ✅ ALL BENCHMARKS COMPLETED!"
    echo ""
    echo "   Next step: Aggregate statistics"
    echo "   python aggregate_statistics.py --results-dir results/config-azure --output-dir paper_statistics"
else
    echo "   ⚠️  INCOMPLETE: Only $COMPLETED/$TOTAL completed"
    echo "   Check logs for errors"
fi
echo ""
echo "======================================"
