#!/bin/bash
# Quick status check for parallel benchmark run

echo "=========================================="
echo "VERUSAGENT PARALLEL RUN STATUS"
echo "=========================================="
echo

# Check if running
PROCESS_COUNT=$(ps aux | grep "run_all_benchmarks.py" | grep -v grep | wc -l)
if [ $PROCESS_COUNT -gt 0 ]; then
    echo "✅ Status: RUNNING"
    echo "   Active processes: $PROCESS_COUNT"
    echo

    # Show latest output
    echo "Latest output (last 10 lines):"
    echo "------------------------------------------"
    tail -10 run_all_benchmarks.out 2>/dev/null || echo "No output yet"
    echo

    # Show log files created
    LOG_COUNT=$(ls logs/*_todo_*.log 2>/dev/null | wc -l)
    echo "Benchmark logs created: $LOG_COUNT / 13"
    if [ $LOG_COUNT -gt 0 ]; then
        echo
        echo "Most recent logs:"
        ls -t logs/*_todo_*.log 2>/dev/null | head -5 | while read log; do
            echo "  - $(basename $log)"
        done
    fi
    echo

    # Show output directories
    OUTPUT_COUNT=$(ls -d output/*_todo 2>/dev/null | wc -l)
    echo "Output directories: $OUTPUT_COUNT / 13"

else
    echo "❌ Status: NOT RUNNING"
    echo

    # Check if completed
    if [ -f run_all_benchmarks.out ]; then
        echo "Checking for completion..."
        if grep -q "SUMMARY" run_all_benchmarks.out; then
            echo "✅ RUN COMPLETED!"
            echo
            tail -30 run_all_benchmarks.out | grep -A 30 "SUMMARY"
        else
            echo "Run was stopped or crashed. Check run_all_benchmarks.out"
        fi
    else
        echo "No run output found. Has the run started?"
    fi
fi

echo
echo "=========================================="
echo "Commands:"
echo "  Monitor output:  tail -f run_all_benchmarks.out"
echo "  Check logs:      ls -lth logs/"
echo "  Check results:   ls -lth output/"
echo "=========================================="
