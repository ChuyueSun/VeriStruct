# VeruSyth

## Class Info

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
    ... # Something that you might want oto add

# Including generate/fix/inference tools, and doc reader 
class Module1(BaseModule):
    system (markdown follows some tempalte)
    instruction (markdown follows some template)
    important note
    
    def exec(context: Context) -> str # Execute the module
```

## Algorithm Flow

Algorithm Flow

1. Planner given context, output Module
   - Module: repair, inference, generate, doc reader
   - Planner can refer to the implementation in https://github.com/henryalps/OpenManus/blob/main/src/prompts/planner.md

2. Execute Module (which may invoke llm) to produce new trial

3. Add trial to context

## Implementation Details:

- Implement `Trial`, `Context`
- Implement planner
- Implement `BaseModule` and each specific Module (Livia)
- Implement complete algorithm flow

## File Tree

```text
archive # the original -verusyth stuffs
examples # rb example here
benchmarks # all vstd benchmarks here
utils # lynette rust implementation
src
- __init__.py # in case you do not want long import name, edit this file
- configs # configs here
  - config-*.json
- main.py # implement the algorithm workflow
- context.py # implement trial and context
- planner.py # implement planner
- infer.py # LLM inference infrastructure
- prompts
  - markdown files used my each module
- modules
  - base.py # implement base module
  - lynette.py
  - houdini.py
  - veval.py # 3 utils that the module may rely on
  - each module corresponds to a file # implement each module
```

VerusAgent 
  --> pass --> Clover
  --> fail --> repair --> Clover

  formal spec: assertions written in logical language
  ?testing: too many to try (infinite input space), mathematically proven for all inputs

## LLM Caching

VerusAgent now includes LLM caching functionality to improve performance and reduce API costs. The cache stores LLM responses based on the query parameters and can be used for subsequent identical requests.

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

Cache files are stored as JSON in the specified cache directory and include:
- Original query parameters
- Response from the LLM
- Timestamp for cache expiration