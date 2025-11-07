# VeriStruct Experimental Evaluation Framework

This directory contains tools and scripts for conducting systematic experimental evaluations of the VeriStruct workflow, following the comprehensive experiment plan outlined in `../EXPERIMENT_PLAN.md`.

## Quick Start

### 1. Prepare Your Benchmark Corpus

Create a JSON file listing your benchmarks (see `sample_corpus.json` for format):

```json
{
  "name": "My Benchmark Corpus",
  "benchmarks": [
    {
      "path": "benchmarks-complete/example.rs",
      "name": "example",
      "category": "simple_data_structures",
      "complexity": "low"
    }
  ]
}
```

### 2. Run Experiments

```bash
# Install required dependencies
pip install pandas numpy scipy matplotlib seaborn

# Run experiment on benchmark corpus
python experiment_runner.py \
  --corpus sample_corpus.json \
  --experiment-name "standard_run_$(date +%Y%m%d)" \
  --config config-azure \
  --output-dir results/ \
  --repair-rounds 5

# For quick testing with limited benchmarks
python experiment_runner.py \
  --corpus sample_corpus.json \
  --experiment-name "test_run" \
  --limit 3
```

### 3. Analyze Results

```bash
# Analyze experimental results
python analyze_results.py \
  --metrics results/your_experiment/your_experiment_metrics.json \
  --output-dir results/your_experiment/analysis/

# View the generated report
cat results/your_experiment/analysis/ANALYSIS_REPORT.md
```

## Directory Structure

```
experiments/
├── README.md                    # This file
├── experiment_runner.py         # Main experiment execution script
├── analyze_results.py           # Statistical analysis and reporting
├── sample_corpus.json           # Example benchmark corpus
├── results/                     # Experiment results (created)
│   └── experiment_name/
│       ├── experiment_name_metrics.json
│       └── analysis/
│           ├── ANALYSIS_REPORT.md
│           ├── analysis_results.json
│           └── *.png (visualizations)
└── configs/                     # Experiment configurations (optional)
    ├── standard.yaml
    ├── ablation_no_repair.yaml
    └── stress_test.yaml
```

## Detailed Usage

### Experiment Runner

The `experiment_runner.py` script automates running VeriStruct on multiple benchmarks and collecting comprehensive metrics.

**Full Options:**

```bash
python experiment_runner.py \
  --corpus CORPUS_FILE \           # Path to benchmark corpus JSON
  --experiment-name NAME \         # Name of experiment (for output files)
  --config CONFIG_NAME \           # VeriStruct config (e.g., config-azure)
  --output-dir DIR \               # Base output directory
  --repair-rounds N \              # Number of repair rounds (default: 5)
  --limit N                        # Limit to N benchmarks (for testing)
```

**What it does:**

- Runs VeriStruct on each benchmark in the corpus
- Collects metrics: robustness, cost, effectiveness
- Handles timeouts (30 minutes per benchmark)
- Saves results to `{experiment_name}_metrics.json`

**Collected Metrics:**

| Category | Metrics |
|----------|---------|
| **Robustness** | Success rate, module completion, error recovery, timeouts |
| **Cost** | Total tokens, API calls, cache hits, time, estimated USD cost |
| **Effectiveness** | Verification success, error reduction, improvement rate |

### Results Analyzer

The `analyze_results.py` script performs statistical analysis and generates comprehensive reports.

**Full Options:**

```bash
python analyze_results.py \
  --metrics METRICS_FILE \         # Metrics JSON from experiment runner
  --output-dir DIR                 # Output directory for analysis
```

**Generated Outputs:**

1. **ANALYSIS_REPORT.md** - Comprehensive markdown report with:
   - Executive summary
   - Robustness analysis
   - Cost analysis
   - Effectiveness analysis
   - Statistical significance tests
   - Recommendations

2. **analysis_results.json** - Structured analysis data

3. **Visualizations** (PNG):
   - `success_by_category.png` - Success rates by benchmark category
   - `cost_distribution.png` - Histogram of costs per benchmark
   - `time_distribution.png` - Histogram of execution times
   - `tokens_vs_time.png` - Scatter plot of token usage vs time
   - `success_pie_chart.png` - Overall success/failure distribution

### Benchmark Corpus Format

A benchmark corpus is a JSON file defining the benchmarks to test:

```json
{
  "name": "Experiment Corpus Name",
  "version": "1.0",
  "description": "Description of the corpus",
  "total_benchmarks": 10,
  "benchmarks": [
    {
      "path": "relative/path/to/benchmark.rs",
      "name": "benchmark_name",
      "category": "category_name",
      "complexity": "low|medium|high",
      "features": ["feature1", "feature2"],
      "expected_difficulty": "easy|medium|hard",
      "notes": "Optional notes"
    }
  ],
  "categories": {
    "category_name": {
      "count": 3,
      "description": "Category description"
    }
  }
}
```

**Categories** (from EXPERIMENT_PLAN.md):

- `simple_data_structures` - Basic data structures
- `complex_data_structures` - Trees, maps, advanced structures
- `algorithms` - Sorting, searching, traversal
- `concurrency` - Atomic operations, concurrent structures
- `edge_cases` - Special patterns, boundary conditions

## Experiment Phases

Following the plan in `../EXPERIMENT_PLAN.md`, experiments are organized into phases:

### Phase 1: Standard Workflow Test

Test all benchmarks with standard configuration:

```bash
python experiment_runner.py \
  --corpus full_corpus.json \
  --experiment-name "phase1_standard" \
  --config config-azure \
  --repair-rounds 5
```

### Phase 2: Ablation Studies

Test individual component contributions by running with different configurations.

**Example: Module Ablation**

You would create multiple runs with different module configurations and compare:

```bash
# Full workflow
python experiment_runner.py --corpus subset.json --experiment-name "ablation_full"

# No view inference (manually modify workflow)
python experiment_runner.py --corpus subset.json --experiment-name "ablation_no_view"

# Compare results
python analyze_results.py --metrics results/ablation_full/metrics.json
python analyze_results.py --metrics results/ablation_no_view/metrics.json
```

### Phase 3: Stress Testing

Test robustness under challenging conditions:

```bash
# Large codebase test
python experiment_runner.py \
  --corpus large_benchmarks.json \
  --experiment-name "stress_large_code"

# Timeout sensitivity
python experiment_runner.py \
  --corpus subset.json \
  --experiment-name "stress_timeout_60min" \
  # (modify timeout in code)
```

### Phase 4: Comparative Evaluation

Compare against baselines or other systems (manual process).

## Example Workflow

Here's a complete example workflow:

```bash
# 1. Create benchmark corpus
cat > my_corpus.json << EOF
{
  "name": "My Test Corpus",
  "benchmarks": [
    {"path": "benchmarks-complete/bitmap_2_todo.rs", "name": "bitmap", "category": "complex"},
    {"path": "benchmarks-complete/vectors.rs", "name": "vectors", "category": "simple"}
  ]
}
EOF

# 2. Run experiment
python experiments/experiment_runner.py \
  --corpus my_corpus.json \
  --experiment-name "my_experiment_$(date +%Y%m%d_%H%M%S)" \
  --config config-azure \
  --output-dir experiments/results/

# 3. Analyze results
LATEST=$(ls -td experiments/results/*/ | head -1)
python experiments/analyze_results.py \
  --metrics ${LATEST}*_metrics.json \
  --output-dir ${LATEST}analysis/

# 4. View report
cat ${LATEST}analysis/ANALYSIS_REPORT.md

# 5. View visualizations
open ${LATEST}analysis/*.png  # macOS
xdg-open ${LATEST}analysis/*.png  # Linux
```

## Metrics Explained

### Robustness Metrics

- **Success Rate**: % of benchmarks that complete without fatal errors
- **Module Completion**: Average number of workflow stages completed
- **Error Recovery Rate**: % of errors successfully repaired
- **Timeout Rate**: % of benchmarks that hit timeout

### Cost Metrics

- **Total Tokens**: Sum of input + output tokens for all LLM calls
- **API Calls**: Number of LLM API requests
- **Cache Hit Rate**: % of requests served from cache (cost savings)
- **Time to Completion**: Wall-clock time per benchmark
- **Estimated Cost**: USD cost based on GPT-4 pricing ($0.03/1K input, $0.06/1K output)

### Effectiveness Metrics

- **Verification Success Rate**: % of benchmarks fully verified (0 errors)
- **Improvement Rate**: % reduction in errors from initial to final
- **Errors Reduced**: Absolute number of errors fixed

## Statistical Analysis

The analyzer performs several statistical tests:

### Hypothesis Testing

**Success Rate Test:**

- H₀: Success rate ≤ 50% (no better than baseline)
- H₁: Success rate > 50%
- Test: One-sample proportion test
- Significance: α = 0.05

### Confidence Intervals

95% confidence intervals are computed for:

- Success rate (binomial confidence interval)
- Mean cost (bootstrap or t-distribution)
- Mean time (t-distribution)

### Comparison Tests

When comparing configurations:

- **Mann-Whitney U test**: Compare distributions (non-parametric)
- **Kruskal-Wallis H test**: Compare >2 groups
- **Paired t-test**: Before/after on same benchmarks

## Tips and Best Practices

### Running Experiments

1. **Start Small**: Test with `--limit 3` before running full corpus
2. **Use Cache**: Ensure `ENABLE_LLM_CACHE=1` to save costs on retries
3. **Monitor Progress**: Check output directory during long runs
4. **Set Budget**: Track `estimated_cost_usd` to avoid surprises

### Corpus Design

1. **Diversity**: Include benchmarks from all categories
2. **Stratified Sampling**: Ensure representative distribution
3. **Difficulty Balance**: Mix easy/medium/hard benchmarks
4. **Known Baselines**: Include benchmarks with known outcomes

### Analysis

1. **Check Sample Size**: Need n≥20 for statistical power
2. **Look for Outliers**: Investigate extremely high/low cases
3. **Category Analysis**: Compare success rates across categories
4. **Cost-Effectiveness**: Balance success rate with cost

## Troubleshooting

### Experiment Runner Issues

**Problem**: `No module named 'src'`
**Solution**: Run from VeriStruct root directory, not experiments/

**Problem**: Timeout on every benchmark
**Solution**: Increase timeout in `experiment_runner.py` or check Verus installation

**Problem**: High cost warnings
**Solution**: Reduce `--repair-rounds`, enable cache, or use `--limit` for testing

### Analysis Issues

**Problem**: "No valid effectiveness data"
**Solution**: Experiments may have failed; check metrics JSON for errors

**Problem**: Visualizations not generated
**Solution**: Install required packages: `pip install matplotlib seaborn pandas`

**Problem**: Empty success_by_category
**Solution**: Ensure benchmarks have `category` field in corpus JSON

## Advanced Usage

### Custom Metrics Collection

To collect additional metrics, extend `ExperimentMetricsCollector` in `experiment_runner.py`:

```python
def collect_run_metrics(self, ...):
    metrics = super().collect_run_metrics(...)

    # Add custom metrics
    metrics["custom"] = {
        "my_metric": calculate_my_metric(context)
    }

    return metrics
```

### Custom Analysis

Create custom analysis scripts using the collected data:

```python
import json
import pandas as pd

# Load metrics
with open('results/experiment/metrics.json') as f:
    data = json.load(f)

df = pd.DataFrame(data)

# Custom analysis
print(df.groupby('category')['cost'].apply(
    lambda x: x.apply(lambda c: c.get('time_seconds', 0)).mean()
))
```

## Contributing

When adding new experiments or analysis:

1. Document the experiment objective
2. Define clear success criteria
3. Follow the metrics schema
4. Add analysis for new metrics
5. Update this README

## References

- **Main Experiment Plan**: `../EXPERIMENT_PLAN.md`
- **VeriStruct Docs**: `../README.md`
- **VEval Scoring**: `../src/modules/veval.py`
- **Repair Modules**: `../src/modules/repair_*.py`

---

**Questions or Issues?**
Contact the VeriStruct team or open an issue in the repository.
