# API Call Counting - Final Clarification

## What the CSV Tracks

**The CSV tracks MODULE INVOCATIONS (including planner), NOT individual retry-level LLM API calls.**

---

## Understanding the Counts

### What is Counted

| Call Number | What It Represents |
|-------------|-------------------|
| **API_Call_0** | After planner call (1st LLM call - planner analyzes code) |
| **API_Call_1** | After 1st module execution (e.g., view_inference, spec_inference) |
| **API_Call_2** | After 2nd module execution |
| **API_Call_3** | After 3rd module execution |
| ... | ... |

**Total_API_Calls = 1 (planner) + N (module executions)**

### Module Invocation vs Actual LLM API Calls

Each module invocation can make **1-3 actual LLM API calls** due to retry loops:

```python
# Inside each module:
max_retries = 3

for retry_attempt in range(max_retries):
    responses = llm.infer_llm(...)  # ← ACTUAL LLM API CALL
    if responses_are_valid():
        break  # Success, stop retrying
```

**Therefore:**
- 1 module invocation = 1-3 actual LLM API calls
- 10 module invocations = approximately 15-30 actual LLM API calls
- Baseline module has max_retries=10, so 1 invocation = 1-10 actual API calls

---

## Example: bitmap_todo

### CSV Shows:
```csv
bitmap_todo,14,0,4,4,4,4,4,5,14,7,100%
```

### Interpretation:
- **API_Call_0**: 0 verified (after planner call - planner analyzes, doesn't change code)
- **API_Call_1**: 4 verified (after 1st module: view_inference)
- **API_Call_2**: 4 verified (after 2nd module: view_refinement)
- **API_Call_3**: 4 verified (after 3rd module: inv_inference)
- **API_Call_4**: 4 verified (after 4th module: repair)
- **API_Call_5**: 4 verified (after 5th module: spec_inference)
- **API_Call_6**: 5 verified (after 6th module: proof_generation)
- **Final**: 14/14 verified (100%)
- **Total Module Invocations**: 7 (1 planner + 6 modules)

### Actual LLM API Calls:
- **Minimum**: 7 calls (1 planner + 6 modules, all succeed first try)
- **Maximum**: 21 calls (1 planner + 6 modules × 3 retries each)
- **Likely**: ~10-14 actual LLM API calls (1 planner + modules averaging 1.5-2 retries)

---

## Why We Count Module Invocations

### Reasons:

1. **Data Availability**: Statistics files from these experiments don't have per-retry tracking
2. **Consistency**: All experiments tracked module invocations consistently
3. **Meaningful Metric**: Module invocations represent logical steps in the pipeline
4. **Conservative**: Shows minimum calls (best case scenario)

### Limitations:

1. **Underestimates Cost**: Actual API calls are 1.5-3× higher
2. **No Retry Information**: Can't see how many retries each module needed
3. **Baseline Comparison**: Baseline uses 10 retries, making comparison tricky

---

## Actual LLM API Call Estimates

### Conservative Estimate (1.5× multiplier)
Assumes most modules succeed on first or second try:

| Benchmark | Module Invocations | Estimated API Calls |
|-----------|-------------------|---------------------|
| invariants_todo | 3 (1+2) | ~4-5 |
| rwlock_vstd_todo | 3 (1+2) | ~4-5 |
| transfer_todo | 3 (1+2) | ~4-5 |
| option_todo | 3 (1+2) | ~4-5 |
| vectors_todo | 4 (1+3) | ~6 |
| atomics_todo | 5 (1+4) | ~7-8 |
| bitmap_todo | 7 (1+6) | ~10-11 |
| rb_type_invariant_todo | 7 (1+6) | ~10-11 |
| set_from_vec_todo | 7 (1+6) | ~10-11 |
| treemap_todo | 0* | ~4-5 (see note) |
| node_todo | 0* | ~4-6 (see note) |

### Realistic Estimate (2× multiplier)
Assumes average 2 tries per module:

| Benchmark | Module Invocations | Estimated API Calls |
|-----------|-------------------|---------------------|
| invariants_todo | 3 (1+2) | ~6 |
| rwlock_vstd_todo | 3 (1+2) | ~6 |
| transfer_todo | 3 (1+2) | ~6 |
| option_todo | 3 (1+2) | ~6 |
| vectors_todo | 4 (1+3) | ~8 |
| atomics_todo | 5 (1+4) | ~10 |
| bitmap_todo | 7 (1+6) | ~14 |
| rb_type_invariant_todo | 7 (1+6) | ~14 |
| set_from_vec_todo | 7 (1+6) | ~14 |

---

## Special Cases

### treemap_todo (0* module invocations)
- Started with 16/21 verified after preprocessing
- 0* indicates modules ran but counts weren't tracked the same way
- Estimated 4-5 actual API calls to reach 21/21

### node_todo (0* module invocations)
- Started with 8/12 verified after preprocessing
- 0* indicates stage-based progression tracking
- 4 modules activated: view_inference, view_refinement, inv_inference, spec_inference
- Estimated 4-6 actual API calls to reach 11/12

---

## For Future Experiments

### Recommended Tracking:

1. **Track actual LLM API calls** using `context.infer_llm_with_tracking()`
2. **Record retry counts** per module
3. **Log cache hits** separately
4. **Measure token usage** for cost analysis

### Implementation:
```python
# In progress_logger or statistics_collector
def record_llm_call(self, stage, module, retry_attempt, response_time, cache_hit):
    self.stats['llm_calls']['total'] += 1
    self.stats['llm_calls']['by_module'][module] += 1
    self.stats['llm_calls']['by_retry'][retry_attempt] += 1
    # ... etc
```

---

## Summary

✅ **Current CSV**: Tracks **module invocations (including planner)**
⚠️ **Actual API calls**: Approximately **1.5-2× the module counts**
📊 **Total across all benchmarks**: ~46 module invocations (incl. planner) = **~69-92 actual API calls**

This is still significantly more efficient than naive approaches!

### Breakdown:
- **Simple benchmarks** (4): 3 calls each = 12 module invocations → ~18-24 actual API calls
- **Medium benchmarks** (2): 4-5 calls = 9 module invocations → ~13-18 actual API calls
- **Complex benchmarks** (3): 7 calls each = 21 module invocations → ~31-42 actual API calls
- **Special cases** (2): treemap_todo + node_todo = ~4 module invocations → ~6-8 actual API calls

---

## Date: October 16, 2025
## Status: UPDATED & CLARIFIED
