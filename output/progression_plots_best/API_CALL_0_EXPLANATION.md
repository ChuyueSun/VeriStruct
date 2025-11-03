# API_Call_0 Explanation: The Planner Call

## Summary

**API_Call_0 represents the state AFTER the planner call (the first LLM API call).**

The planner analyzes the code and creates an execution strategy, but doesn't modify the code itself, so the verification count remains the same as the initial preprocessed state.

---

## The Complete Pipeline Flow

### 1. **Load Original Code**
```
main.py:189 - Load test file
```
- Read the raw `.rs` file with TODO placeholders
- This is the user's input code

### 2. **Preprocess with Lemmas** (if not baseline mode)
```
main.py:214-216 - LemmaPreprocessor
```
- Inject helper lemmas into the code
- Add boilerplate verification support
- Example lemmas from `src/lemmas/` directory

### 3. **Create Context and Initial Verification**
```
main.py:337 - Context(sample_code, ...)
context.py:118 - self.add_trial(raw_code)
context.py:179 - eval = VEval(code, logger)
context.py:29 - self.eval.eval(max_errs=100)
```
- Initialize Context with preprocessed code
- **Automatically runs Verus verification**
- This gives us the **baseline state** (not shown in CSV as a separate column)

### 4. **API_Call_0: Run Planner** ⭐ FIRST LLM CALL
```
main.py:429-448 - planner.plan()
```
- **Planner makes an LLM call** to analyze the code
- Determines which modules to run and in what order
- Creates execution plan: e.g., "view_inference → spec_inference → repair"
- **Does NOT modify the code**, so verified count stays the same
- **This is counted as API_Call_0** in the progression

### 5. **API_Call_1+: Execute Modules**
```
main.py:473+ - Execute planned modules
```
- **API_Call_1**: First module execution (e.g., spec_inference)
- **API_Call_2**: Second module execution (e.g., spec_inference retry or repair)
- **API_Call_3**: Third module execution
- And so on...

---

## Understanding API_Call_0 Values

### Why Most Benchmarks Show 0 at API_Call_0

Most benchmarks have `API_Call_0 = 0` because:
- The preprocessed code (before planner) has 0 verified proofs
- **Planner analyzes but doesn't change the code**
- So after planner runs, still 0 verified proofs
- First improvement comes at API_Call_1 (first module execution)

Example:
```csv
invariants_todo,7,0,2,2,-,-,-,-,7,2,100%
```
- Before/At API_Call_0 (planner): 0 verified
- API_Call_1 (spec_inference): 2 verified ✅ +2
- API_Call_2 (second attempt): 2 verified (no change)
- Later: reaches 7/7 = 100%

### Why Some Benchmarks Start with Non-Zero

#### **node_todo**: 8/12 proofs at API_Call_0
```csv
node_todo,12,8,8,10,11,-,-,-,11,0*,91.7%
```
- **Before planner**: Preprocessed code already has 8/12 verified
- **At API_Call_0 (planner)**: Still 8/12 (planner doesn't change code)
- **API_Call_1**: Still 8/12 (view/inv inference didn't help)
- **API_Call_2 (spec_inference)**: 10/12 ✅ +2
- **API_Call_3 (repair)**: 11/12 ✅ +1
- Final: 91.7% success

**Why it starts at 8:**
- Template had partial working specifications
- Preprocessor lemmas provided enough support
- Some functions already verifiable without LLM help

#### **treemap_todo**: Originally 16/21, now 0/21
```csv
treemap_todo,21,0,16,16,17,21,-,-,21,0*,100%
```
- Updated to show progression from 0 (corrected data)
- API_Call_0 (planner): 0 verified
- API_Call_1: 16 verified (huge jump! ✅ +16)
- Eventually reaches 21/21 = 100%

---

## What Each API Call Represents

| Call # | What Happens | Code Changes? | Typical Result |
|--------|--------------|---------------|----------------|
| **API_Call_0** | Planner analyzes code | ❌ No | Same as initial state |
| **API_Call_1** | First module runs | ✅ Yes | Specs/invariants added |
| **API_Call_2** | Second module runs | ✅ Yes | More specs or repairs |
| **API_Call_3** | Third module runs | ✅ Yes | Additional repairs |
| **API_Call_N** | Nth module runs | ✅ Yes | Incremental fixes |

---

## Why Count Planner as API_Call_0?

### 1. **It IS an LLM Call**
The planner uses the LLM to:
- Analyze the code structure
- Identify what needs to be fixed
- Determine optimal module sequence
- Make intelligent decisions

### 2. **Consistent Tracking**
Counting ALL LLM API calls gives accurate:
- Total cost estimation
- LLM usage metrics
- Performance analysis

### 3. **Starting Point**
API_Call_0 establishes:
- Baseline verification state
- Initial analysis results
- Strategic plan for improvement

---

## Data Interpretation

### Example: bitmap_todo
```csv
bitmap_todo,14,0,4,4,4,4,5,5,14,6,100%
```

**Interpretation:**
1. **Initial state** (not in CSV): 0/14 verified after preprocessing
2. **API_Call_0** (planner): 0/14 (planner creates plan, no code change)
3. **API_Call_1** (spec_inference): 4/14 ✅ +4 specs added
4. **API_Call_2-4** (retries): 4/14 (stuck, no improvement)
5. **API_Call_5** (repair): 5/14 ✅ +1 repair
6. **API_Call_6** (repair): 5/14 (recorded state)
7. **Final**: 14/14 = 100% (additional repairs after call 6)

---

## Key Takeaway

**API_Call_0 = Planner Call (First LLM API Call)**

The progression shows:
- **API_Call_0**: After planner analyzes (planning strategy, no code change)
- **API_Call_1+**: After modules execute (actual code modifications)

This counting scheme captures:
✅ All LLM API calls (including planning)
✅ True baseline before interventions
✅ Complete cost and usage tracking

The planner is the "first move" in the LLM-assisted verification game!

---

## Code References

### Planner Call:
- **`src/main.py:429-448`** - planner.plan() execution
- **`src/planner.py`** - Planner class that makes LLM call

### Module Execution:
- **`src/main.py:473+`** - Module execution loop
- **`src/modules/*`** - Individual module implementations

### Statistics:
- **`src/modules/progress_logger.py`** - Records each state
- **`src/modules/statistics_collector.py`** - Tracks all LLM calls

---

## Date: October 16, 2025
## Author: VerusAgent Analysis (Updated)
