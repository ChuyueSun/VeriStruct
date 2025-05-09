# VerusAgent

VerusAgent is an AI-powered assistant for Verus formal verification. It helps develop, debug, and refine Rust code with formal specifications.

## Architecture Overview

```python
class Trial:
    verus_code
    veval_result

class Context:
    cur_trial: Trial # Current trial
    prev_trials: List[Trial] # Previous failing Trials
    knowledgebase: Dict[str(e.g., LocalInvariant, PCM), str(code in vstd)]
    # The knowledge LLM has been queried so far

class BaseModule:
    hdn # houdini algorithm
    example # examples here
    default_system # default system prompt
    ... # Something that you might want to add

# Including generate/fix/inference tools, and doc reader
class Module1(BaseModule):
    system (markdown follows some tempalte)
    instruction (markdown follows some template)
    important note

    def exec(context: Context) -> str # Execute the module
```

## Algorithm Flow

1. Planner given context, output Module
   - Module: repair, inference, generate, doc reader
   - Planner can refer to the implementation in https://github.com/henryalps/OpenManus/blob/main/src/prompts/planner.md

2. Execute Module (which may invoke LLM) to produce new trial

3. Add trial to context

## Implementation Details

- Implements `Trial`, `Context`
- Implements planner
- Implements `BaseModule` and each specific Module
- Implements complete algorithm flow

## File Structure

```text
.
├── README.md                     # Main documentation
├── MIGRATION_PLAN.md             # Migration plan documentation
├── README_modules.md             # Module-specific documentation
├── README_progress_logger.md     # Documentation for progress logging
├── run.sh                        # Main execution script (with fish shell)
├── run_with_options.sh           # Execution script with additional options
├── disable_llm_run.sh            # Run in dummy mode without LLM API calls
├── customize.fish                # Environment customization script
├── .gitignore                    # Git ignore file
├── .pre-commit-config.yaml       # Pre-commit hook configuration
├── archive/                      # Archive of older files
├── benchmarks/                   # Verus benchmark files
├── examples/                     # Example files for testing
│   ├── input-view-refine/        # Input examples for view refinement
│   └── output-view-refine/       # Expected outputs for view refinement
├── output/                       # Output directory for generated files
│   ├── best/                    # Best verification results
│   ├── debug/                   # Debug information
│   ├── progress_logs/           # Progress logs
│   └── samples/                 # Intermediate samples
├── llm_cache/                   # LLM response cache storage (.json files) and prompts (.md files)
├── tmp/                         # Temporary files directory
└── utils/                        # Utility code directory
```

## Workflow

VerusAgent follows this workflow:
```
VerusAgent
  --> pass --> Clover (code verification successful)
  --> fail --> repair --> Clover
```

The system works with formal specifications (assertions written in logical language) rather than testing. This approach mathematically proves correctness for all inputs, rather than just testing a subset of possible inputs.

## LLM Caching

VerusAgent includes LLM caching functionality to improve performance and reduce API costs. The cache stores LLM responses based on the query parameters and can be used for subsequent identical requests.

### Configuration

LLM caching is controlled by the following environment variables:

- `ENABLE_LLM_CACHE`: Set to `1` to enable caching for reading (default is enabled)
- `LLM_CACHE_DIR`: Directory to store cache files (default is `llm_cache` in the project root)
- `LLM_CACHE_MAX_AGE_DAYS`: Maximum age of cache entries in days (default is 7 days)

These variables are set in the `run.sh` script.

### Cache Writing Behavior

By default, the system will write to the cache even when cache reading is disabled. This allows you to build up a cache of responses over time. This behavior can be controlled via the `always_write_cache` configuration option.

### Testing Cache Functionality

To test the LLM cache with Azure configuration:

```bash
# Run with a unique query (will trigger an API call)
fish test_azure_cache.fish

# Run with a fixed query to see caching in action
fish test_azure_cache.fish --fixed-query

# Run the fixed query test again to see cache hits on both calls
fish test_azure_cache.fish --fixed-query
```

The first run with a fixed query will cache the response, and subsequent runs will retrieve from cache without making API calls.

### Cache Files

Cache files are stored in the specified cache directory:
- JSON files (.json): Contain the LLM responses, original query parameters, and cache timestamp
- Markdown files (.md): Contain the full prompts sent to the LLM with the same base filename as the corresponding JSON response

This dual storage provides transparency in debugging by allowing you to examine both the exact prompt sent to the LLM and its response.

## Environment Customization

VerusAgent can be customized for different development environments using environment variables. This is particularly useful for settings that vary between machines, such as project directories and Verus executable paths.

### Using the Customization Script

The repository includes a `customize.fish` script that you can edit for your specific environment:

1. Edit `customize.fish` and set your preferred values:
   ```fish
   # Set your project directory
   set -x VERUS_PROJECT_DIR "/path/to/your/project"
   
   # Set your Verus executable path
   set -x VERUS_PATH "/path/to/your/verus/executable"
   
   # Optionally set a custom test file
   set -x VERUS_TEST_FILE "/path/to/your/test_file.rs"
   ```

2. Run the agent with your custom settings:
   ```bash
   source customize.fish && ./run.sh
   ```

### Available Environment Variables

- `VERUS_PROJECT_DIR`: Base directory for finding benchmarks, examples, and other resources
- `VERUS_PATH`: Path to your Verus executable
- `VERUS_TEST_FILE`: Optional custom test file to use instead of the default

These settings can also be set directly in your shell before running the agent:

```bash
export VERUS_PROJECT_DIR="/path/to/your/project"
export VERUS_PATH="/path/to/your/verus/executable"
./run.sh
```

### Default Behavior

If not explicitly set:
- `VERUS_PROJECT_DIR` defaults to the VerusAgent repository root directory
- `VERUS_PATH` is read from the configuration file
- The default test file specified in `main.py` is used

## Getting Started

1. Clone the repository
2. Configure your environment (edit `customize.fish` or set environment variables)
3. Run `./run.sh` to start the agent

## Modules

VerusAgent includes multiple specialized modules for different tasks:

- **View Inference**: Generates a View function for a data structure (mathematical abstraction)
- **View Refinement**: Improves the mathematical abstraction for data structures
- **Invariant Inference**: Generates invariants that capture data structure properties
- **Error Repair Modules**:
  - Assertion Repair
  - Arithmetic Repair
  - Decrease Repair
  - Invariant Repair
  - Missing Implementation Repair
  - Mode Repair
  - Postcondition Repair
  - Precondition Repair
  - Syntax Repair
  - Type Repair
- **Specification Inference**: Generates specifications for code
- **Progress Logger**: Tracks and reports on verification progress