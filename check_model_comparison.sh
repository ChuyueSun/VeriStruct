#!/bin/bash
# Monitor model comparison progress

COMP_DIR=$(ls -dt model_comparison_* 2>/dev/null | head -1)

if [ -z "$COMP_DIR" ]; then
    echo "No model comparison found"
    exit 1
fi

echo "=========================================="
echo "Model Comparison: $COMP_DIR"
echo "=========================================="
echo ""

# Check running processes
RUNNING=$(ps aux | grep "run_agent.py" | grep "node_todo" | grep -v grep | wc -l)
echo "Active processes: $RUNNING"
echo ""

# Check each model status
for config in azure-o1mini azure-o3mini anthropic-sonnet; do
    log="$COMP_DIR/${config}.log"

    if [ -f "$log" ]; then
        lines=$(wc -l < "$log")

        if grep -q "Verification Success: Yes" "$log" 2>/dev/null; then
            status="✅ SUCCESS"
        elif grep -q "Verification Success: No" "$log" 2>/dev/null; then
            status="⚠️ PARTIAL"
        elif [ "$lines" -gt 500 ]; then
            status="🔄 RUNNING ($lines lines)"
        elif [ "$lines" -gt 10 ]; then
            status="🔄 STARTING ($lines lines)"
        else
            status="⏳ PENDING"
        fi

        printf "%-25s %s\n" "$config" "$status"
    else
        printf "%-25s %s\n" "$config" "⏳ NOT STARTED"
    fi
done

echo ""

# Show output directories created
echo "Output directories created:"
for dir in output/node_todo/{azure_o1mini,azure_o3mini,anthropic_sonnet}_*/; do
    if [ -d "$dir" ]; then
        echo "  - $(basename $(dirname $dir))/$(basename $dir)"
    fi
done 2>/dev/null

echo ""
echo "To watch live: watch -n 5 $0"
