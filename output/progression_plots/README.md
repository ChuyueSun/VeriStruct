# Verification Progression Visualizations

**Date**: October 14, 2025
**Generated**: 19 visualizations

## Overview

These plots show how verified functions progress through the repair pipeline as LLM calls accumulate. Each point represents a verification check after an LLM call (either an initial inference stage or a repair attempt).

## Plot Types

### 1. Per-Benchmark Comparison Plots (11 files)

Files: `progression_<benchmark_name>.png`

Shows all 7 top experiments for a single benchmark:
- **X-axis**: Cumulative number of LLM calls
- **Y-axis**: Number of verified functions at that point
- **Green dashed line**: Ground truth (total functions in completed benchmark)
- **Multiple colored lines**: Different experiments

These plots reveal:
- Which experiments reached full verification for this benchmark
- How quickly each experiment converged
- Whether experiments got stuck at local maxima

**Examples**:
- `progression_atomics_todo.png` - 6/7 experiments reached full verification (11/11)
- `progression_option_todo.png` - 6/7 experiments reached full verification (15/15)
- `progression_node_todo.png` - 0/7 experiments made any progress (0/12)

### 2. All-Benchmarks-Per-Experiment Plots (7 files)

Files: `all_benchmarks_repair_experiment_<timestamp>.png`

Shows all 11 benchmarks for a single experiment in a 3x4 grid:
- Each subplot shows one benchmark's progression
- **Green background**: Benchmark reached full verification
- **White background**: Partial verification or no progress
- **Title**: Shows max verified / ground truth

These plots reveal:
- Overall performance pattern of an experiment
- Which benchmarks are easy/hard for that configuration
- Visual comparison of success across benchmarks

**Best Performer**: `all_benchmarks_repair_experiment_20251011_160518.png`
- 7/11 benchmarks fully verified (green backgrounds)

### 3. Aggregate Statistics Plot (1 file)

File: `aggregate_statistics.png`

Four subplots showing cross-experiment statistics:

1. **Total Verified Functions**: Bar chart of summed verified functions across all benchmarks
2. **Fully Verified Benchmarks**: Count of benchmarks reaching ground truth
3. **Average LLM Calls per Benchmark**: Efficiency metric
4. **Overall Completion Percentage**: Ratio of verified to ground truth

## Key Findings

### Experiments Ranked by Full Verification Count:
1. **repair_experiment_20251011_160518**: 7/11 benchmarks (64%)
2. **repair_experiment_20251011_135504**: 6/11 benchmarks (55%)
3. **repair_experiment_20251010_152504**: 6/11 benchmarks (55%)
4. Others: 4-5 benchmarks

### Always Fully Verified:
- `invariants_todo` (7 functions) - All experiments succeeded
- `rwlock_vstd_todo` (5 functions) - All experiments succeeded

### Never Fully Verified:
- `bitmap_todo` (15 functions) - Best: 8/15 (53%)
- `bst_map_todo` (21 functions) - Best: 10/21 (48%)
- `node_todo` (12 functions) - No progress in any experiment
- `vectors_todo` (16 functions) - Best: 7/16 (44%)

### Typical Progression Pattern:

Most successful benchmarks follow this pattern:
1. **0 LLM calls**: 0 verified (initial TODO state)
2. **Calls 1-5**: Initial inference stages (view, invariant, spec, proof)
   - Gradual increase in verified functions
   - May plateau at intermediate values
3. **Calls 6+**: Repair rounds
   - Incremental fixes of remaining errors
   - Often reaches full verification within 10-15 total calls
   - Some get stuck and make no further progress

### LLM Call Efficiency:

- **Fast convergers** (5-8 calls): `rwlock_vstd_todo`, `transfer_todo`, `invariants_todo`
- **Moderate** (8-15 calls): `atomics_todo`, `option_todo`, `rb_type_invariant_todo`
- **Slow/partial** (15+ calls): `bitmap_todo`, `vectors_todo`, `bst_map_todo`
- **No progress**: `node_todo` (needs different approach)

## How to Interpret

### Good Signs:
- ✅ Steady upward progression
- ✅ Reaches ground truth line
- ✅ Few plateaus or drops
- ✅ Converges quickly (< 10 calls)

### Problem Signs:
- ⚠️ Long plateaus (stuck at local maximum)
- ⚠️ Oscillations (repairs making things worse)
- ⚠️ Horizontal from start (no initial progress)
- ⚠️ Gap remains to ground truth despite many calls

### What This Tells Us:

1. **Simple benchmarks** (5-7 functions) are often fully verified within 5-8 LLM calls
2. **Medium benchmarks** (11-15 functions) need 10-15 calls and may plateau
3. **Complex benchmarks** (16-21 functions) rarely reach full verification
4. **Initial inference stages** (first 3-5 calls) do most of the work
5. **Repair rounds** help but often get stuck

## Files Location

All plots saved to: `/home/chuyue/VerusAgent/output/progression_plots/`

## Reproduction

To regenerate these plots:
```bash
python3 plot_verification_progression.py
```

Data source: Progress JSON files in each experiment's `progress_logs/` directory
