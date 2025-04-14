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
