# VeruSyth

## Class Info

class Trial:
    verus_code
    veval_result

class Context:
    cur_trial: Trial
    prev_trials: List[Trial]
    knowledgebase: Dict[str(e.g., LocalInvariant, PCM), str(code in vstd)]

class BaseModule:
    hdn
    example
    default_system


class Module1(BaseModule):
    system (markdown follows some tempalte)
    instruction (markdown follows some template)
    important note
    knowledgebase
    context info

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
code
- main.py # implement the algorithm workflow
- context.py # implement trial and context
- planner.py # implement planner
- prompts
  - markdown files used my each module
- modules
  - base.py # implement base module
  - each module corresponds to a file # implement each module
```
