#!/bin/bash
# Quick experiment launcher for VerusAgent testing
# Usage: ./run_quick_experiment.sh [experiment_name] [num_benchmarks]

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
EXPERIMENT_NAME="${1:-quick_test_$(date +%Y%m%d_%H%M%S)}"
NUM_BENCHMARKS="${2:-5}"
CONFIG="config-azure"
REPAIR_ROUNDS=5

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        VerusAgent Quick Experiment Launcher               ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo ""
echo -e "${GREEN}Experiment Name:${NC} $EXPERIMENT_NAME"
echo -e "${GREEN}Benchmarks:${NC} $NUM_BENCHMARKS (from sample corpus)"
echo -e "${GREEN}Config:${NC} $CONFIG"
echo -e "${GREEN}Repair Rounds:${NC} $REPAIR_ROUNDS"
echo ""

# Check dependencies
echo -e "${YELLOW}[1/5] Checking dependencies...${NC}"
python3 -c "import pandas, numpy, scipy, matplotlib, seaborn" 2>/dev/null || {
    echo -e "${RED}ERROR: Required Python packages not found${NC}"
    echo "Install with: pip install pandas numpy scipy matplotlib seaborn"
    exit 1
}
echo -e "${GREEN}✓ Dependencies OK${NC}"

# Check sample corpus exists
CORPUS_FILE="$SCRIPT_DIR/sample_corpus.json"
if [ ! -f "$CORPUS_FILE" ]; then
    echo -e "${RED}ERROR: Sample corpus not found at $CORPUS_FILE${NC}"
    exit 1
fi

# Create results directory
RESULTS_DIR="$SCRIPT_DIR/results/$EXPERIMENT_NAME"
mkdir -p "$RESULTS_DIR"
echo -e "${GREEN}✓ Results directory: $RESULTS_DIR${NC}"

# Step 2: Run experiment
echo ""
echo -e "${YELLOW}[2/5] Running experiment...${NC}"
echo -e "${BLUE}This may take a while. Timeout: 30 minutes per benchmark${NC}"

cd "$ROOT_DIR"
python3 "$SCRIPT_DIR/experiment_runner.py" \
    --corpus "$CORPUS_FILE" \
    --experiment-name "$EXPERIMENT_NAME" \
    --config "$CONFIG" \
    --output-dir "$SCRIPT_DIR/results" \
    --repair-rounds "$REPAIR_ROUNDS" \
    --limit "$NUM_BENCHMARKS" || {
    echo -e "${RED}ERROR: Experiment failed${NC}"
    exit 1
}

echo -e "${GREEN}✓ Experiment completed${NC}"

# Step 3: Find metrics file
echo ""
echo -e "${YELLOW}[3/5] Locating metrics file...${NC}"
METRICS_FILE="$RESULTS_DIR/${EXPERIMENT_NAME}_metrics.json"

if [ ! -f "$METRICS_FILE" ]; then
    echo -e "${RED}ERROR: Metrics file not found: $METRICS_FILE${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Found metrics: $METRICS_FILE${NC}"

# Step 4: Analyze results
echo ""
echo -e "${YELLOW}[4/5] Analyzing results...${NC}"
ANALYSIS_DIR="$RESULTS_DIR/analysis"
mkdir -p "$ANALYSIS_DIR"

python3 "$SCRIPT_DIR/analyze_results.py" \
    --metrics "$METRICS_FILE" \
    --output-dir "$ANALYSIS_DIR" || {
    echo -e "${RED}ERROR: Analysis failed${NC}"
    exit 1
}

echo -e "${GREEN}✓ Analysis completed${NC}"

# Step 5: Display summary
echo ""
echo -e "${YELLOW}[5/5] Generating summary...${NC}"

# Extract key metrics from JSON
if command -v jq &> /dev/null; then
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                    QUICK RESULTS SUMMARY                   ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

    # Count successes
    TOTAL=$(jq 'length' "$METRICS_FILE")
    SUCCESS=$(jq '[.[] | select(.robustness.success == true)] | length' "$METRICS_FILE")

    if [ "$TOTAL" -gt 0 ]; then
        SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($SUCCESS/$TOTAL)*100}")
        echo -e "${GREEN}Success Rate:${NC} $SUCCESS/$TOTAL benchmarks ($SUCCESS_RATE%)"
    fi

    # Average time
    AVG_TIME=$(jq '[.[] | .cost.time_seconds] | add / length / 60' "$METRICS_FILE" 2>/dev/null)
    if [ ! -z "$AVG_TIME" ]; then
        echo -e "${GREEN}Average Time:${NC} $(printf "%.1f" $AVG_TIME) minutes per benchmark"
    fi

    # Total cost
    TOTAL_COST=$(jq '[.[] | .cost.estimated_cost_usd // 0] | add' "$METRICS_FILE" 2>/dev/null)
    if [ ! -z "$TOTAL_COST" ]; then
        echo -e "${GREEN}Total Cost:${NC} \$$(printf "%.2f" $TOTAL_COST)"
    fi

    echo ""
fi

# Show file locations
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                     OUTPUT FILES                           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo -e "${GREEN}📊 Analysis Report:${NC}"
echo "   $ANALYSIS_DIR/ANALYSIS_REPORT.md"
echo ""
echo -e "${GREEN}📈 Visualizations:${NC}"
echo "   $ANALYSIS_DIR/*.png"
echo ""
echo -e "${GREEN}📋 Raw Metrics:${NC}"
echo "   $METRICS_FILE"
echo ""

# Offer to open report
echo -e "${YELLOW}View full report? (y/n)${NC}"
read -t 10 -n 1 response || response="n"
echo ""

if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
    REPORT_FILE="$ANALYSIS_DIR/ANALYSIS_REPORT.md"

    # Try different markdown viewers
    if command -v glow &> /dev/null; then
        glow "$REPORT_FILE"
    elif command -v mdless &> /dev/null; then
        mdless "$REPORT_FILE"
    elif command -v bat &> /dev/null; then
        bat "$REPORT_FILE"
    else
        less "$REPORT_FILE"
    fi
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║            ✓ EXPERIMENT COMPLETE!                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Results saved to: ${BLUE}$RESULTS_DIR${NC}"
echo ""

# Cleanup suggestion
echo -e "${YELLOW}Tip:${NC} To run another experiment with different settings, use:"
echo "  ./run_quick_experiment.sh my_experiment_name 10"
echo ""
