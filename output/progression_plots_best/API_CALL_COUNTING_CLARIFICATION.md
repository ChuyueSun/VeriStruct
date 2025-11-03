# API Call Counting Clarification

## The Problem

**The CSV currently tracks MODULE INVOCATIONS, not actual LLM API CALLS.**

Each module can make **multiple LLM API calls** due to retry mechanisms!

---

## How Modules Actually Work

### Retry Mechanism (Found in Code)

Every module has a retry loop:

```python
max_retries = 3  # Most modules (spec_inference, inv_inference, etc.)
# OR
max_retries = 10  # Baseline module only

for retry_attempt in range(max_retries):
    responses = self._get_llm_responses(...)  # ← ACTUAL LLM API CALL
    safe_responses = self._process_responses(responses, original_code)

    if safe_responses:
        # Success! Stop retrying
        break

    # Failed safety checks, try again
```

**Result**:
- 1 module invocation = **1 to 3 actual LLM API calls** (most modules)
- 1 baseline invocation = **1 to 10 actual LLM API calls**

### Additional Complexity

Each LLM API call generates multiple samples:
- `answer_num = 3` for most modules (spec_inference, inv_inference, etc.)
- `answer_num = 5` for baseline
- These samples are generated in a SINGLE API call

---

## What Should Be Counted

### Current CSV Counting (INCORRECT for true API calls)
```csv
bitmap_todo,14,0,4,4,4,4,5,5,14,6,100%
```

This shows:
- **API_Call_1**: 4 verified (but this might be after view_inference made 1-3 actual calls)
- **API_Call_2**: 4 verified (after view_refinement made 1-3 actual calls)
- etc.

### True LLM API Call Count

For **bitmap_todo** according to `detailed_llm_call_analysis.json`:
1. **Module 1**: view_inference
   - Could make: 1-3 actual LLM calls (retries)
2. **Module 2**: view_refinement
   - Could make: 1-3 actual LLM calls
3. **Module 3**: inv_inference
   - Could make: 1-3 actual LLM calls
4. **Module 4**: repair_assertion
   - Could make: 1-3 actual LLM calls
5. **Module 5**: spec_inference
   - Could make: 1-3 actual LLM calls
6. **Module 6**: proof_generation
   - Could make: 1-3 actual LLM calls
7. **Module 7**: repair_postcond
   - Could make: 1-3 actual LLM calls

**Total**: 7 module invocations = **7 to 21 actual LLM API calls!**

---

## What the CSV Should Track

### Option 1: True LLM API Call Count (Recommended)
Count every call to `llm.infer_llm()`:
- Planner: 1 call
- view_inference retry 1: 1 call
- view_inference retry 2: 1 call (if needed)
- view_inference retry 3: 1 call (if needed)
- view_refinement retry 1: 1 call
- etc.

### Option 2: Module Invocation Count (Current)
Count each time a module is invoked (regardless of retries):
- Planner: 1 invocation
- view_inference: 1 invocation (even if it made 3 LLM calls internally)
- view_refinement: 1 invocation
- etc.

---

## Recommendation

**The CSV should be updated to track ACTUAL LLM API CALLS**, not module invocations, because:

1. **Cost**: API costs are per call, not per module
2. **Accuracy**: "LLM API calls" means actual API requests
3. **Comparison**: Need true numbers to compare with other systems
4. **Transparency**: Users need to know the real API usage

---

## How to Get True Count

Check the statistics files for actual LLM call counts:

```python
# In statistics/detailed_*.json or statistics/summary_*.json
{
  "llm_calls": {
    "total": 25,  # ← TRUE count of actual API calls
    "by_stage": {...},
    "by_module": {...},
    "response_times": [...]
  }
}
```

The `progress_logger.record_llm_call()` function should be called for EACH actual LLM API call, not just once per module.

---

## Example Correction Needed

### Current CSV (Module Invocations)
```csv
bitmap_todo,14,0,4,4,4,4,5,5,14,6,100%
                    ↑
            6 module invocations
```

### Should Be (Actual LLM Calls)
```csv
bitmap_todo,14,0,4,4,4,4,5,5,14,15,100%
                              ↑↑
                    Could be 7-21 actual LLM calls
```

Need to check the statistics files to get the exact count!

---

## Action Items

1. ✅ Review statistics files for each benchmark
2. ✅ Extract `llm_calls.total` from each run
3. ✅ Update CSV with true API call counts
4. ✅ Recalculate all aggregations and plots
5. ✅ Update documentation to clarify counting methodology

---

## Date: October 16, 2025
## Status: NEEDS CORRECTION
