# Baseline Mode for VerusAgent (New-Workflow Branch)

This document explains how to use the baseline mode functionality that provides a single-shot LLM approach for comparison with the multi-stage pipeline on the new-workflow branch.

## Overview

The baseline mode skips the sophisticated multi-stage pipeline (planner → spec_inference → view_inference → inv_inference → repairs) and instead uses a single comprehensive LLM call to generate both specifications and proofs at once.

## Implementation Architecture

### Core Components

#### 1. **BaselineModule** (`src/modules/baseline.py`)
- **Purpose**: Single-shot specification and proof generation
- **Integration**: Inherits from `BaseModule`, uses existing `LLM` and `VEval` infrastructure
- **Features**:
  - Comprehensive instruction covering all verification tasks
  - Multiple candidate generation (5 per attempt)
  - Retry logic with temperature escalation (0.7, 0.8, 0.9)
  - Safety checking for immutable functions
  - VEval scoring integration

#### 2. **Main Integration** (`src/main.py`)
- **Environment Detection**: Checks `VERUS_BASELINE_MODE=1` flag
- **Pipeline Bypass**: Skips planner and multi-stage execution
- **Progress Integration**: Uses existing `ProgressLogger` system
- **Output Consistency**: Maintains same file structure as regular pipeline

#### 3. **Batch Execution** (`run_baseline_bench.py`)
- **Automation**: Processes all `*_todo.rs` files automatically
- **Statistics**: Comprehensive performance tracking and reporting
- **Flexibility**: Multiple configs, timeouts, benchmark limits
- **Error Handling**: Graceful failure management and recovery

## Usage Guide

### Single Benchmark Execution
```bash
# Set environment variables
export VERUS_TEST_FILE="benchmarks-complete/rb_type_invariant_todo.rs"
export VERUS_CONFIG="config-azure"
export VERUS_OUTPUT_DIR="baseline_output"
export VERUS_BASELINE_MODE="1"

# Run VerusAgent in baseline mode
python -m src.main
```

### Batch Benchmark Execution
```bash
# Quick test run (2 benchmarks, 3-minute timeout)
./run_baseline_bench.py --max-benchmarks 2 --timeout 3

# Full benchmark suite with default settings
./run_baseline_bench.py

# Custom configuration
./run_baseline_bench.py \
  --configs config-azure config-gpt4 \
  --output-dir my-baseline-results \
  --benchmark-dir benchmarks-complete \
  --timeout 20
```

### System Integration Test
```bash
# Verify baseline system setup
./test_baseline_simple.py
```

## Output Structure

```
results-baseline/
├── config-azure/                          # Results per configuration
│   ├── bst_map_todo/                      # Per-benchmark directory
│   │   ├── baseline_output.log            # Full execution log
│   │   ├── 01_baseline_bst_map_todo__*.rs # Generated code with VEval score
│   │   ├── samples/                       # Raw LLM samples
│   │   │   ├── baseline_raw_sample_*.rs   # Individual LLM responses
│   │   │   └── ...
│   │   ├── best/                          # Best results directory
│   │   │   ├── best_bst_map_todo.rs      # Best result for this benchmark
│   │   │   └── best.rs                   # Standardized best result
│   │   └── checkpoint_best_*.rs          # Checkpoint best with metadata
│   └── ...
├── statistics/                            # Aggregated statistics
│   ├── config-azure_detailed_stats.json  # Individual benchmark stats
│   ├── config-azure_summary_stats.json   # Summary statistics
│   └── config-azure_report.txt           # Human-readable report
└── verification_plan_*.txt                # Would contain plan (bypassed in baseline)
```

## Key Features

### Comprehensive Verification Instruction
The baseline module uses a single instruction that covers:
- **Specifications**: `requires`/`ensures` clauses, `spec fn` implementations
- **Invariants**: Data structure invariants, loop invariants
- **Proofs**: Proof blocks, assertions, ghost variables, lemma calls
- **Views**: `View` trait implementations for data structures
- **Safety**: Immutable function protection, type safety

### Advanced Error Handling
- **Timeout Management**: Configurable per-benchmark timeouts
- **Retry Logic**: Multiple attempts with increasing randomness
- **Safety Checking**: Validates code changes don't violate constraints
- **Graceful Degradation**: Returns original code if generation fails

### Statistics Collection
Tracks comprehensive metrics:
- **Success Rates**: Verification success per benchmark
- **Performance**: Execution times, timeout rates
- **Quality**: VEval scores, error analysis
- **Output**: Generated file counts, log sizes

## Comparison Framework

### Baseline vs Multi-Stage Pipeline

| **Aspect** | **Baseline Mode** | **Multi-Stage Pipeline** |
|------------|-------------------|---------------------------|
| **Approach** | Single comprehensive LLM call | AI planner + specialized modules |
| **Instruction** | "Complete all verification tasks" | Module-specific prompts |
| **Refinement** | None (single-shot) | Iterative between stages |
| **Examples** | General baseline examples | Stage-specific examples |
| **Repair** | None | Sophisticated error repair modules |
| **Planning** | No planner | AI planner determines execution order |
| **Execution Time** | Fast (single call) | Slower (multiple stages) |
| **Success Rate** | Expected lower | Expected higher |
| **Code Quality** | Variable | More consistent |

### Performance Metrics
The baseline provides comparison data for:
- **Effectiveness**: Success rates and verification quality
- **Efficiency**: Time and computational resource usage
- **Robustness**: Performance across different complexity levels
- **Scalability**: Handling of diverse verification challenges

## Environment Configuration

### Required Environment Variables
- **`VERUS_BASELINE_MODE=1`**: Enables baseline mode execution
- **`VERUS_TEST_FILE`**: Path to the benchmark file to process
- **`VERUS_CONFIG`**: Configuration file name (e.g., "config-azure")
- **`VERUS_OUTPUT_DIR`**: Output directory for results and logs

### Optional Environment Variables
- **`VERUS_IMMUTABLE_FUNCTIONS`**: Comma-separated list of protected functions
- **`ENABLE_LLM_INFERENCE`**: Set to "0" to disable LLM calls (for testing)
- **`LOG_LEVEL`**: Logging verbosity ("DEBUG", "INFO", "ERROR")

## Research Applications

### Academic Value
The baseline system enables rigorous academic evaluation:
- **Quantitative Comparison**: Objective metrics for approach effectiveness
- **Ablation Studies**: Measuring individual component contributions
- **Benchmark Standardization**: Consistent evaluation across different systems
- **Reproducible Results**: Documented methodology and configurations

### Development Applications
- **Performance Baselines**: Establish minimum performance thresholds
- **Regression Testing**: Verify that pipeline improvements provide real benefits
- **Module Evaluation**: Test new components against established baselines
- **System Optimization**: Identify bottlenecks and improvement opportunities

## Troubleshooting

### Common Issues and Solutions

#### **Import Errors**
```bash
# Error: ModuleNotFoundError: No module named 'loguru'
# Solution: Install dependencies in proper environment
pip install loguru pathlib typing
```

#### **Configuration Errors**
```bash
# Error: Config file not found
# Solution: Verify config exists
ls src/configs/config-azure.json
```

#### **Permission Errors**
```bash
# Error: Permission denied
# Solution: Make scripts executable
chmod +x run_baseline_bench.py test_baseline_simple.py
```

#### **Timeout Issues**
```bash
# Error: Benchmarks timing out
# Solution: Increase timeout or reduce benchmark set
./run_baseline_bench.py --timeout 30 --max-benchmarks 5
```

### Debugging Options
```bash
# Enable verbose logging
export LOG_LEVEL="DEBUG"

# Disable LLM calls for testing
export ENABLE_LLM_INFERENCE="0"

# Run system integration test
./test_baseline_simple.py
```

## Advanced Usage

### Custom Baseline Instructions
Modify `src/modules/baseline.py` to customize the baseline instruction:
```python
self.baseline_instruction = """
Your custom comprehensive instruction here...
Focus on specific verification aspects...
"""
```

### Multiple Configuration Testing
```bash
# Test multiple LLM configurations
./run_baseline_bench.py --configs config-azure config-gpt4 config-claude
```

### Selective Benchmark Testing
```bash
# Test specific benchmark patterns
./run_baseline_bench.py \
  --benchmark-dir benchmarks-complete \
  --pattern "*invariant*_todo.rs"
```

### Statistics Analysis
```python
# Load and analyze statistics programmatically
import json
with open("results-baseline/statistics/config-azure_detailed_stats.json") as f:
    stats = json.load(f)
# Perform custom analysis...
```

## Integration with Existing Workflow

### Compatibility
- **Branch**: Designed for new-workflow branch architecture
- **Dependencies**: Uses existing `src/` infrastructure
- **Configurations**: Compatible with all existing config files
- **Output**: Maintains consistency with regular pipeline output

### Testing Integration
```bash
# Test baseline, then regular pipeline
export VERUS_BASELINE_MODE="1"
python -m src.main  # Baseline execution

unset VERUS_BASELINE_MODE
python -m src.main  # Regular pipeline execution
```

## Future Enhancements

### Planned Improvements
- **Dynamic Instructions**: Adapt baseline instruction based on code analysis
- **Incremental Baseline**: Multi-shot baseline with limited refinement
- **Hybrid Approaches**: Combine baseline with selective pipeline stages
- **Advanced Statistics**: Code quality metrics, error pattern analysis

### Research Extensions
- **Comparative Studies**: Systematic comparison with other verification approaches
- **Human Evaluation**: Expert assessment of generated proof quality
- **Benchmark Expansion**: Additional verification challenges and domains
- **Performance Optimization**: Efficiency improvements for large-scale deployment

---

The baseline system provides a robust foundation for comparing single-shot LLM approaches with sophisticated multi-stage verification pipelines, enabling rigorous academic evaluation and system development on the new-workflow branch.
