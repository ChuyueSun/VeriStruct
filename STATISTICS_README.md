# VerusAgent Statistics Collection System

This document describes the comprehensive statistics collection system for VerusAgent research experiments.

## Overview

The statistics system tracks detailed metrics during VerusAgent execution for research paper reporting:

1. **Number of LLM calls** per stage/module
2. **Number of iterations** in each module
3. **Number of modules activated**
4. **Types of repair heuristics** triggered
5. **Response times** for each operation
6. **Success rates** and error distributions
7. **Code metrics** and verification results

## Components

### 1. StatisticsCollector (`src/modules/statistics_collector.py`)

Collects detailed statistics during execution:
- Module activation tracking
- LLM call counting and timing
- Iteration tracking per module
- Repair heuristic usage
- Error type distributions
- Initial and final state comparison

### 2. Enhanced ProgressLogger (`src/modules/progress_logger.py`)

Integrates statistics collection with existing progress logging:
- Automatically tracks stages, repairs, and LLM calls
- Saves detailed JSON statistics
- Generates human-readable reports

### 3. Statistics Aggregator (`aggregate_statistics.py`)

Aggregates statistics across multiple benchmark runs:
- CSV summary tables
- LaTeX tables for research papers
- Aggregate statistical analysis
- Success rate calculations

## Usage

### Running Experiments with Statistics

Statistics are automatically collected when running benchmarks:

```bash
# Run a single benchmark
./run_agent.py --test-file benchmarks-complete/my_benchmark_todo.rs

# Run all benchmarks
python run_bench.py --configs config-azure
```

Statistics are saved in the `output/statistics/` directory for each run.

### File Outputs

For each benchmark run, the following files are generated:

1. **Detailed Statistics** (`detailed_<benchmark>_<timestamp>.json`)
   - Complete statistics in JSON format
   - All LLM calls with timing
   - All repair attempts with results
   - Module activation sequence

2. **Summary Statistics** (`summary_<benchmark>_<timestamp>.json`)
   - Condensed key metrics
   - Suitable for quick analysis

3. **Human-Readable Report** (`report_<benchmark>_<timestamp>.txt`)
   - Formatted text report
   - Easy to read and share

### Aggregating Results

After running multiple benchmarks, aggregate the statistics:

```bash
# Aggregate all results from the results directory
python aggregate_statistics.py --results-dir results --output-dir aggregate_results

# Or specify a custom results directory
python aggregate_statistics.py --results-dir output --output-dir paper_stats
```

This generates:

1. **Summary Table** (`summary_table.csv`)
   - CSV format for spreadsheets
   - Per-benchmark metrics

2. **LaTeX Table** (`summary_table.tex`)
   - Ready for inclusion in research papers
   - Formatted with booktabs

3. **Aggregate Report** (`aggregate_report.txt`)
   - Statistical analysis across all benchmarks
   - Mean, median, standard deviation
   - Success rates and error distributions

4. **Aggregate JSON** (`aggregate_statistics.json`)
   - All aggregate data in JSON format
   - For further analysis

## Statistics Tracked

### Per-Benchmark Statistics

#### Execution Metrics
- Total execution time
- Time per stage/module
- Time per repair round

#### LLM Call Metrics
- Total number of LLM calls
- LLM calls per stage
- LLM calls per module
- Response time per call
- Cache hit rate

#### Module Activation
- List of modules activated
- Total number of modules
- Iterations per module

#### Repair Metrics
- Total repair rounds
- Total repair attempts
- Repairs by error type
- Repairs by heuristic/module
- Success rate per repair type
- Time per repair

#### Error Metrics
- Initial error count
- Final error count
- Errors by type
- Errors fixed by type

#### Verification Results
- Initial verified count
- Final verified count
- Compilation errors
- Verification success (yes/no)

#### Code Metrics
- Initial code length
- Final code length
- Code changes (lines modified)

### Aggregate Statistics

#### Overall Summary
- Total benchmarks processed
- Successfully verified count
- Overall success rate

#### Execution Time Statistics
- Mean execution time
- Median execution time
- Standard deviation
- Min/Max execution time
- Total time across all benchmarks

#### LLM Call Statistics
- Total LLM calls across all benchmarks
- Mean calls per benchmark
- Median calls per benchmark
- Cache hit rate
- LLM calls by stage (aggregated)

#### Response Time Statistics
- Mean response time
- Median response time
- Min/Max response time

#### Module Activation Statistics
- Mean modules per benchmark
- Median modules per benchmark
- Most frequently used modules

#### Repair Statistics
- Total repair rounds
- Total repair attempts
- Overall repair success rate
- Most frequently used heuristics
- Repair heuristic effectiveness

#### Error Statistics
- Total initial errors
- Total final errors
- Total errors fixed
- Overall error fix rate
- Error type distribution

## Example Output

### Detailed Statistics (JSON)

```json
{
  "benchmark_name": "rb_type_invariant_todo",
  "total_execution_time": 45.32,
  "modules_activated": ["spec_inference", "repair_type", "repair_assertion"],
  "modules_count": 3,
  "llm_calls": {
    "total": 12,
    "by_stage": {
      "spec_inference": 5,
      "repair_type": 4,
      "repair_assertion": 3
    },
    "cache_hits": 2,
    "cache_misses": 10
  },
  "repairs": {
    "total_rounds": 2,
    "total_repairs": 7,
    "successful_repairs": 5,
    "repairs_by_heuristic": {
      "repair_type": 4,
      "repair_assertion": 3
    }
  }
}
```

### Summary Table (CSV)

```
Benchmark,Exec Time (s),Modules,LLM Calls,Repairs,Success Rate %,Initial Errors,Final Errors,Verified
rb_type_invariant_todo,45.32,3,12,7,71.4,8,2,No
vec_spec_todo,32.15,2,8,5,100.0,5,0,Yes
...
```

### LaTeX Table

```latex
\begin{table}[htbp]
\centering
\caption{VerusAgent Performance on Benchmarks}
\label{tab:verus-performance}
\begin{tabular}{lrrrrrrr}
\toprule
Benchmark & Time(s) & Modules & LLM Calls & Repairs & Success\% & Init Err & Final Err \\
\midrule
rb\_type\_invariant & 45.3 & 3 & 12 & 7 & 71.4 & 8 & 2 \\
vec\_spec & 32.2 & 2 & 8 & 5 & 100.0 & 5 & 0 \\
\bottomrule
\end{tabular}
\end{table}
```

## Advanced Usage

### Custom Analysis

You can load and analyze the JSON files directly:

```python
import json
from pathlib import Path

# Load detailed statistics
with open("output/statistics/detailed_benchmark_20250104_120000.json") as f:
    stats = json.load(f)

# Analyze LLM calls per stage
llm_by_stage = stats["llm_calls"]["by_stage"]
print(f"LLM calls in spec_inference: {llm_by_stage.get('spec_inference', 0)}")

# Analyze repair success rate
repairs = stats["repairs"]
success_rate = repairs["successful_repairs"] / repairs["total_repairs"] * 100
print(f"Repair success rate: {success_rate:.1f}%")
```

### Filtering Benchmarks

You can filter benchmarks before aggregation:

```python
from aggregate_statistics import StatisticsAggregator

aggregator = StatisticsAggregator(Path("results"))
aggregator.collect_statistics()

# Filter only successful benchmarks
successful_benchmarks = [
    b for b in aggregator.benchmarks
    if b["verification"]["final_errors"] == 0
]

print(f"Successful benchmarks: {len(successful_benchmarks)}")
```

## Integration with Research Papers

### Tables

Use the generated LaTeX tables directly in your paper:

```latex
\section{Experimental Results}

We evaluated VerusAgent on XX benchmarks. Table~\ref{tab:verus-performance}
shows the performance results.

\input{aggregate_results/summary_table.tex}
```

### Statistics

Reference the aggregate statistics in your analysis:

- "VerusAgent achieved an overall success rate of XX% across YY benchmarks"
- "The average execution time was XX seconds (median: YY seconds)"
- "The system made an average of XX LLM calls per benchmark"
- "Repair heuristics were successful in XX% of attempts"

### Graphs

Generate graphs from the JSON data using your preferred plotting library (matplotlib, seaborn, etc.):

```python
import json
import matplotlib.pyplot as plt

with open("aggregate_results/aggregate_statistics.json") as f:
    stats = json.load(f)

# Plot LLM calls by stage
stages = list(stats["llm_by_stage"].keys())
counts = list(stats["llm_by_stage"].values())

plt.bar(stages, counts)
plt.xlabel("Stage")
plt.ylabel("LLM Calls")
plt.title("LLM Calls by Stage")
plt.xticks(rotation=45)
plt.tight_layout()
plt.savefig("llm_calls_by_stage.pdf")
```

## Troubleshooting

### No Statistics Generated

If statistics are not being generated:

1. Check that `output/statistics/` directory exists
2. Verify that the `ProgressLogger` is initialized correctly
3. Check log files for error messages

### Aggregation Fails

If aggregation fails to find benchmarks:

1. Verify the results directory path
2. Check that statistics files exist: `find results -name "detailed_*.json"`
3. Ensure JSON files are valid: `python -m json.tool <file>`

### Missing Metrics

If certain metrics are not tracked:

1. Ensure the module uses `context.infer_llm_with_tracking()` instead of `context.llm.infer_llm()`
2. Verify that stages call `progress_logger.start_step()` and `progress_logger.end_step()`
3. Check that repairs use `progress_logger.add_repair()`

## Future Enhancements

Potential future additions to the statistics system:

1. **Memory usage tracking**: Track peak memory usage per stage
2. **Token counting**: Track input/output tokens for LLM calls
3. **Proof complexity metrics**: Measure complexity of generated proofs
4. **Comparative analysis**: Compare against baseline systems
5. **Interactive dashboard**: Web-based dashboard for visualizing statistics
6. **Real-time monitoring**: Monitor statistics during long-running experiments

## Contact

For questions or issues with the statistics system, please open an issue on the repository.
