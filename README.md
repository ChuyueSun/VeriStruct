# VerusAgent

**An AI-Powered Assistant for Verus Formal Verification**

VerusAgent is an automated system that helps develop, debug, and refine Rust code with Verus formal specifications. It uses Large Language Models (LLMs) to generate specifications, infer invariants, and repair verification errors.

---

## 🎯 Overview

VerusAgent automates the challenging process of formal verification by:

- **Generating specifications** (preconditions, postconditions, invariants)
- **Inferring mathematical abstractions** (View functions)
- **Detecting and repairing verification errors** automatically
- **Learning from examples** in the knowledge base
- **Iteratively improving** code until verification succeeds

### Key Features

✅ **Automated Specification Inference**: Generates requires/ensures clauses
✅ **View Function Generation**: Creates mathematical abstractions for data structures
✅ **Invariant Inference**: Discovers data structure invariants
✅ **Smart Error Repair**: 14+ specialized repair modules for different error types
✅ **Timeout Protection**: Automatic timeout detection and retry mechanisms
✅ **LLM Caching**: Reduces API costs and improves response times
✅ **Comprehensive Statistics**: Tracks performance metrics for research

---

## 🚀 Quick Start

### Prerequisites

- Python 3.8+
- Verus (install from [verus-lang.github.io](https://verus-lang.github.io))
- LLM API access (OpenAI, Azure, Anthropic, or DeepSeek)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/VerusAgent.git
cd VerusAgent

# Install dependencies
pip install -r requirements.txt

# Configure your LLM API
cp src/configs/config.json.template src/configs/config.json
# Edit config.json with your API keys and settings
# See src/configs/README.md for detailed configuration instructions
```

### Running VerusAgent

```bash
# Run on a single file
python run_agent.py --test-file benchmarks-complete/vectors_todo.rs

# Run on all benchmarks
python run_all_benchmarks.py --configs config-azure

# Run with custom configuration
python run_bench.py --config src/configs/config-azure.json --test-file my_file.rs
```

---

## 📚 Architecture

### Core Components

```
┌─────────────┐
│   Planner   │  ← Decides which module to execute
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│          Modules                    │
│  • Spec Inference                   │
│  • View Inference                   │
│  • Invariant Inference              │
│  • Repair Modules (12 types)        │
│  • Proof Generation                 │
└──────┬──────────────────────────────┘
       │
       ▼
┌─────────────┐
│   Verus     │  ← Verifies the code
└─────────────┘
```

### Workflow

```
Input Code (incomplete/buggy)
    ↓
Spec Inference → Generate specs
    ↓
Verus Verification
    ↓
    ├─→ ✅ Success → Done
    │
    └─→ ❌ Errors → Planner → Select Repair Module
                         ↓
                    Fix Errors
                         ↓
                    Retry Verification
                         ↓
                    (Iterate until success or max retries)
```

---

## 🧩 Modules

VerusAgent includes specialized modules for different verification tasks:

### Inference Modules

| Module | Description |
|--------|-------------|
| **Spec Inference** | Generates preconditions and postconditions for functions |
| **View Inference** | Creates View functions (mathematical abstractions) for data structures |
| **View Refinement** | Improves existing View functions |
| **Invariant Inference** | Generates invariant functions for complex data structures |
| **Proof Generation** | Generates proof code (assert/assume statements) |

### Repair Modules

| Module | Fixes |
|--------|-------|
| **Assertion Repair** | Invalid assertions |
| **Arithmetic Repair** | Integer overflow/underflow |
| **Decrease Repair** | Termination proofs (decreases clauses) |
| **Invariant Repair** | Loop invariants |
| **Missing Repair** | Missing requires/ensures/invariants |
| **Mode Repair** | exec/proof/spec mode errors |
| **Old-Self Repair** | Incorrect use of `old()` |
| **Postcondition Repair** | Invalid ensures clauses |
| **Precondition Repair** | Invalid requires clauses |
| **Remove Invariant** | Over-specified invariants |
| **Syntax Repair** | Verus syntax errors |
| **Test Assertion Repair** | Failed test assertions |
| **Type Repair** | Type mismatches |
| **Regex Repair** | Pattern-based error fixes |

See [`documentation/technical/modules/`](documentation/technical/modules/) for detailed documentation.

---

## 📂 Project Structure

```
VerusAgent/
├── src/                              # Source code
│   ├── modules/                      # Module implementations
│   │   ├── spec_inference.py         # Specification generation
│   │   ├── proof_generation.py       # Proof code generation
│   │   ├── repair_*.py               # Repair modules
│   │   └── ...
│   ├── prompts/                      # LLM prompt templates
│   ├── configs/                      # Configuration files
│   ├── examples/                     # Example inputs/outputs for learning
│   ├── main.py                       # Main entry point
│   └── planner.py                    # Module selection logic
│
├── benchmarks/                       # Original benchmarks
├── benchmarks-complete/              # Complete (verified) benchmarks
├── benchmarks-too-complicated/       # Complex benchmarks
│
├── output/                           # Experiment results and analysis
│   ├── atomics_todo/                 # Results for atomics benchmark
│   ├── vectors_todo/                 # Results for vectors benchmark
│   └── ...
│
├── documentation/                    # Comprehensive documentation
│   ├── technical/                    # Technical design docs
│   │   ├── modules/                  # Per-module documentation
│   │   └── workflow.md               # System workflow
│   └── tutorial/                     # Getting started guides
│
├── tests/                            # Test files
├── utils/                            # Utility scripts
│
├── run_agent.py                      # Run on single file
├── run_all_benchmarks.py             # Run on all benchmarks
├── run_bench.py                      # Run with specific config
│
└── README.md                         # This file
```

---

## ⚙️ Configuration

Configuration files are in `src/configs/`. Key settings:

### LLM Configuration

```json
{
  "aoai_api_key": "your-api-key",
  "aoai_generation_model": "gpt-4",
  "aoai_api_base": "https://api.openai.com/v1",
  "aoai_api_version": "2023-05-15"
}
```

### Available Configurations

- `config-azure.json` - Azure OpenAI
- `config-oai.json` - OpenAI
- `config-anthropic.json` - Anthropic Claude
- `config-deepseek.json` - DeepSeek

### Environment Variables

```bash
# Optional customization
export VERUS_PATH="/path/to/verus"
export ENABLE_LLM_CACHE=1
export LLM_CACHE_DIR="llm_cache"
```

---

## 🧪 Benchmarks

VerusAgent includes multiple benchmark suites:

| Benchmark | Description | Functions |
|-----------|-------------|-----------|
| `vectors_todo` | Dynamic array with Vec | 8 |
| `bitmap_todo` | Bitmap data structure | 11 |
| `bitmap_2_todo` | Extended bitmap operations | 11 |
| `node_todo` | Linked list node | 9 |
| `bst_map_todo` | Binary search tree map | 11 |
| `treemap_todo` | Tree map data structure | 12 |
| `atomics_todo` | Atomic operations | 6 |
| `option_todo` | Option type wrapper | 5 |
| `rb_type_invariant_todo` | Ring Buffer | 12 |
| `transfer_todo` | State transfer protocol | 7 |
| `rwlock_vstd_todo` | Read-write lock | 8 |
| `set_from_vec_todo` | Set from vector | 6 |
| `invariants_todo` | Various invariants | 10 |

### Running Benchmarks

```bash
# Run all benchmarks in parallel
fish run_all_benchmarks_parallel.fish

# Run specific benchmark
python run_agent.py --test-file benchmarks-complete/vectors_todo.rs

# Run with specific configuration
python run_bench.py --config config-azure --benchmark vectors_todo
```

---

## 📊 Statistics & Analysis

VerusAgent collects comprehensive statistics for research:

- **LLM call counts** per stage/module
- **Iteration counts** and convergence metrics
- **Repair success rates** by error type
- **Execution times** and performance metrics
- **Verification outcomes** (success/failure)

See [`STATISTICS_README.md`](STATISTICS_README.md) for details.

### Generating Reports

```bash
# Statistics are automatically collected during runs
python run_all_benchmarks.py --configs config-azure

# View results in output/ directory
# - JSON files: Raw statistics
# - CSV files: Summary tables
# - MD files: Analysis reports
```

---

## 🔧 Advanced Features

### LLM Caching

Reduce API costs and improve performance:

```bash
# Enable caching (default)
export ENABLE_LLM_CACHE=1

# Set cache directory
export LLM_CACHE_DIR="llm_cache"

# Set cache expiration (days)
export LLM_CACHE_MAX_AGE_DAYS=7
```

Cache files are stored as:
- `.json` - LLM responses with metadata
- `.md` - Original prompts for debugging

### Custom Examples

Add domain-specific examples to improve results:

1. Add input example: `src/examples/input-proof/my_example.rs`
2. Add output example: `src/examples/output-proof/my_example.rs`
3. Examples are automatically matched and used by modules

### Custom Repair Modules

Create specialized repair modules:

```python
from src.modules.baserepair import BaseRepairModule

class MyRepairModule(BaseRepairModule):
    ERROR_TYPE = "my_error_pattern"

    def exec(self, context):
        # Your repair logic
        return repaired_code
```

Register in `src/modules/repair_registry.py`.

---

## 📖 Documentation

### Getting Started
- **README.md** (this file) - Overview and quick start

### Technical Documentation
- [`README_modules.md`](README_modules.md) - Module overview
- [`STATISTICS_README.md`](STATISTICS_README.md) - Statistics system

### Research & Results
- [`README_BASELINE.md`](README_BASELINE.md) - Baseline experiments
- [`output/`](output/) - Experimental results and analysis

---

---


## 📧 Contact

For questions or issues, please open an issue on GitHub.

---

## 🔗 Related Projects

- [Verus](https://github.com/verus-lang/verus) - A verification system for Rust

---

**Happy Verifying! 🚀**
