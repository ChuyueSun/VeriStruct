# VeriStruct Modules

This repository contains modules for automatic verification of Verus code.

## Modules Implemented

1. **ViewInferenceModule**: Generates a View function for a data structure, which is a mathematical abstraction used in specifications.
2. **ViewRefinementModule**: Improves an existing View function to make it more suitable as an abstraction.
3. **InvInferenceModule**: Generates an inv function that captures all necessary invariants of a data structure.

## Running the System

There are two ways to run the system:

### 1. With LLM API Calls

This requires valid API keys for OpenAI or other LLM providers:

```bash
./run.sh
```

### 2. Without LLM API Calls (For Testing)

This uses a dummy implementation that returns placeholder responses:

```bash
./disable_llm_run.sh
```

## Configuration

Configuration is stored in `src/configs/config-verusagent.json`. Key settings:

- `example_path`: Path to the examples directory
- `aoai_api_key`: Your API key(s) for LLM access
- `aoai_generation_model`: The model to use for code generation

## Project Structure

- `src/modules/`: Contains the module implementations
- `src/prompts/`: Contains templates for prompts
- `src/configs/`: Contains configuration files
- `examples/`: Contains example Verus code (input) and their solutions (output)
- `output/`: Where results are saved
- `tests/`: Contains test Verus files

## Example Output

When running the system, it will:

1. Generate a View function from the input code
2. Refine the View function for better abstraction
3. Generate an inv function to capture data structure invariants
4. Save all intermediate and final results in the output directory
