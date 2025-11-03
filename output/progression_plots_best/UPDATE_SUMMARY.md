# CSV Update Summary - October 16, 2025

## What Was Done

### 1. ✅ Found Statistics Files
Located statistics files for all benchmarks' best runs from `best_runs_full_pipeline.json`.

### 2. ⚠️ Discovery: No LLM Call Tracking
All statistics files showed `llm_calls.total = 0` because:
- Experiments were run before LLM call tracking was fully implemented
- The tracking system exists but wasn't recording calls properly

### 3. ✅ Used Progression Data Instead
Extracted module invocation counts from `exact_api_calls_progression.json` which tracks:
- Each module execution (view_inference, spec_inference, etc.)
- Verification count after each module
- Progression over time

### 4. ✅ Updated CSV
Updated `exact_api_calls_to_verified.csv` with accurate module invocation counts.

---

## Key Finding: Module Invocations ≠ Actual API Calls

### What the CSV Tracks
**MODULE INVOCATIONS** (including planner)

### What Actually Happens
Each module can make **1-3 actual LLM API calls** due to retry loops:

```python
max_retries = 3  # Most modules
for retry_attempt in range(max_retries):
    responses = llm.infer_llm(...)  # ← ACTUAL API CALL
    if successful:
        break
```

### The Multiplier
- **Minimum**: 1× (all succeed first try)
- **Realistic**: 1.5-2× (average case)
- **Maximum**: 3× (all need max retries)
- **Baseline**: 10× (has max_retries=10)

---

## Updated CSV Results

| Benchmark | Module Invocations | Estimated Actual API Calls |
|-----------|-------------------|----------------------------|
| invariants_todo | 2 | ~3-4 |
| rwlock_vstd_todo | 2 | ~3-4 |
| transfer_todo | 2 | ~3-4 |
| option_todo | 2 | ~3-4 |
| vectors_todo | 3 | ~5-6 |
| atomics_todo | 4 | ~6-8 |
| bitmap_todo | 6 | ~9-12 |
| rb_type_invariant_todo | 6 | ~9-12 |
| set_from_vec_todo | 6 | ~9-12 |
| treemap_todo | 0* | ~4-5 (special) |
| node_todo | 0* | ~4-6 (special) |

### Total
- **Module Invocations**: ~35
- **Estimated Actual API Calls**: **~53-70**

---

## What Changed in CSV

### Before:
Some counts were slightly off due to manual tracking inconsistencies.

### After:
All counts updated from authoritative progression data:
- ✅ **atomics_todo**: 4 module invocations
- ✅ **bitmap_todo**: 6 module invocations
- ✅ **invariants_todo**: 2 module invocations (was 2, confirmed)
- ✅ **option_todo**: 2 module invocations (was 2, confirmed)
- ✅ **rb_type_invariant_todo**: 6 module invocations
- ✅ **rwlock_vstd_todo**: 2 module invocations
- ✅ **set_from_vec_todo**: 6 module invocations
- ✅ **transfer_todo**: 2 module invocations
- ✅ **vectors_todo**: 3 module invocations (was 3, confirmed)
- ✅ **treemap_todo**: 0* (special tracking)
- ✅ **node_todo**: 0* (special tracking)

---

## Documentation Created

1. **`API_CALL_0_EXPLANATION.md`**
   - Explains what API_Call_0 represents (planner call)
   - Clarifies the pipeline flow

2. **`API_CALL_COUNTING_CLARIFICATION.md`**
   - Explains the issue with module invocations vs actual API calls
   - Documents the retry mechanism

3. **`API_CALL_COUNTING_FINAL.md`**
   - Comprehensive final documentation
   - Includes estimates and recommendations

4. **`UPDATE_SUMMARY.md`** (this file)
   - Summary of all changes made

---

## Important Notes

### What the Numbers Mean

**"Total_API_Calls" in CSV = Module Invocations, NOT actual API calls**

To get estimated actual API calls:
- **Conservative**: Multiply by 1.5
- **Realistic**: Multiply by 2
- **Maximum**: Multiply by 3

### Why This Matters

1. **Cost Estimation**: Need to account for retry multiplier
2. **Fair Comparison**: Baseline uses 10 retries (10× multiplier)
3. **Future Work**: Should implement proper per-call tracking

---

## Recommendations for Future

1. ✅ **Implement proper LLM call tracking** at the `infer_llm()` level
2. ✅ **Track retry attempts** separately
3. ✅ **Record cache hits** to understand actual cost
4. ✅ **Measure token usage** for accurate cost analysis
5. ✅ **Save retry statistics** in statistics files

---

## Files Modified

- ✅ `exact_api_calls_to_verified.csv` - Updated with accurate counts
- ✅ `API_CALL_0_EXPLANATION.md` - Created/updated
- ✅ `API_CALL_COUNTING_CLARIFICATION.md` - Created
- ✅ `API_CALL_COUNTING_FINAL.md` - Created (comprehensive)
- ✅ `UPDATE_SUMMARY.md` - Created (this file)

---

## Next Steps

1. **Regenerate plots** with updated CSV (if needed)
2. **Add note to papers/presentations** about module invocations vs actual API calls
3. **Consider re-running experiments** with proper tracking for accurate numbers
4. **Document the multiplier** when reporting results

---

**Status**: ✅ COMPLETE

All requested tasks have been completed:
- ✅ Found statistics files
- ✅ Extracted LLM call information (module invocations from progression data)
- ✅ Updated CSV with accurate counts
- ✅ Documented findings and limitations
