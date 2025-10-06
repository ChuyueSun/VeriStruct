# Baseline Mode for VerusAgent

This document explains how to use the baseline mode functionality that provides a single-shot LLM approach for comparison with the multi-stage pipeline.

## Overview

The baseline mode skips the sophisticated multi-stage pipeline (spec_inference → view_inference → inv_inference → repairs) and instead uses a single comprehensive LLM call to generate both specifications and proofs at once.

## Files Created

### 1. `src/modules/baseline.py`
- **BaselineModule**: Implements single-shot specification and proof generation
- Uses comprehensive instruction that asks LLM to complete all TODOs in one call
- Generates multiple candidates and picks the best scoring one
- Includes retry logic with increasing temperature

### 2. `run_baseline_bench.py`
- **Batch script**: Runs baseline mode on all benchmark files
- Creates `results-baseline/` directory structure
- Supports multiple configs (e.g., config-azure, config-gpt4)
- Generates logs and results for each benchmark

### 3. Modified `src/main.py`
- **Baseline mode detection**: Checks `VERUS_BASELINE_MODE=1` environment variable
- **Pipeline bypass**: Skips planner and multi-stage execution when in baseline mode
- **Direct execution**: Runs only the BaselineModule

## Usage

### Running Single Baseline
```bash
# Set environment variables for baseline mode
export VERUS_TEST_FILE="benchmarks-complete/rb_type_invariant_todo.rs"
export VERUS_CONFIG="config-azure"
export VERUS_OUTPUT_DIR="baseline_output"
export VERUS_BASELINE_MODE="1"

# Run VerusAgent in baseline mode
python -m src.main
```

### Running Baseline Benchmark Suite
```bash
# Run baseline on all benchmarks with default config
./run_baseline_bench.py

# Run with specific configs
./run_baseline_bench.py --configs config-azure config-gpt4

# Customize output directory and benchmark location
./run_baseline_bench.py --output-dir my-baseline-results --benchmark-dir benchmarks-complete
```

## Output Structure

```
results-baseline/
├── config-azure/                    # Results for each config
│   ├── rb_type_invariant_todo/      # Each benchmark gets its own directory
│   │   ├── baseline_output.log      # Full execution log
│   │   ├── 01_baseline_*.rs         # Intermediate result with score
│   │   └── final_result.rs          # Final generated code
│   ├── basic_lock1_todo/
│   │   └── ...
│   └── ...
└── config-gpt4/                     # Results for other configs
    └── ...
```

## Key Differences from Multi-Stage Pipeline

| Aspect | Baseline Mode | Multi-Stage Pipeline |
|--------|---------------|----------------------|
| **Approach** | Single comprehensive LLM call | Multiple specialized modules |
| **Instruction** | "Complete all TODOs" | Specific per module (specs, views, invariants) |
| **Refinement** | None (single-shot) | Iterative refinement between stages |
| **Examples** | General baseline examples | Stage-specific examples |
| **Repair** | None | Sophisticated error repair modules |
| **Planning** | No planner | AI planner determines execution order |

## Baseline Module Features

### Comprehensive Instruction
The baseline module uses a single instruction that asks the LLM to:
- Add requires/ensures clauses to functions
- Implement missing invariant functions (`inv`, `well_formed`)
- Add View implementations for data structures
- Insert loop invariants for all loops
- Add proof blocks with assertions and lemma calls
- Add ghost variables where helpful

### Multiple Candidates
- Generates 5 candidates per attempt
- Evaluates each with VEval scoring
- Selects the best scoring candidate
- Early termination if correct solution found

### Retry Logic
- Up to 3 retry attempts
- Increases temperature on retries (0.7, 0.8, 0.9)
- Disables cache for retry attempts

### Safety Checking
- Respects immutable functions constraint
- Uses existing code safety infrastructure
- Skips unsafe candidates

## Environment Variables

- `VERUS_BASELINE_MODE=1`: Enables baseline mode
- `VERUS_TEST_FILE`: Path to the _todo.rs file to process
- `VERUS_CONFIG`: Config file to use (e.g., "config-azure")
- `VERUS_OUTPUT_DIR`: Directory for output files
- `VERUS_IMMUTABLE_FUNCTIONS`: Comma-separated list of functions not to modify

## Expected Results

The baseline approach serves as a performance benchmark to measure:
- **Success rate**: How often single-shot LLM generates correct proofs
- **Quality**: VEval scores compared to multi-stage pipeline
- **Efficiency**: Time and token usage for direct approach
- **Robustness**: Handling of different verification tasks

Use baseline results to demonstrate the value of the sophisticated multi-stage pipeline approach.