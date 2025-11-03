# ⚠️ UPDATED Analysis - Module Invocations (NOT API Calls)

## Critical Update

**Previous analysis incorrectly used "LLM calls" when referring to "module invocations".**

## What Changed

### ❌ Previous (INCORRECT)
- "184 total LLM API calls"
- "Each module makes 1 LLM call"
- "invariants_todo succeeded with 1 LLM call"

### ✅ Corrected (ACCURATE)
- "184 total module invocations"
- "Each module can make 1-3 API calls (due to retry loops)"
- "invariants_todo succeeded with 1 module invocation (= 1-3 actual API calls)"

---

## Key Facts

### Module Retry Mechanism
```python
max_retries = 3  # Standard for most modules
for retry_attempt in range(max_retries):
    responses = llm.infer_llm(...)  # ACTUAL API CALL
    if success:
        break
```

**Result**: 1 module invocation can = 1-3 actual LLM API calls

### What We Actually Measured

| Measurement | Count | Meaning |
|-------------|-------|---------|
| **Module Invocations** | 184 | Times modules were called |
| **Actual API Calls** | Unknown | Est. 200-550 (1-3× per invocation) |

---

## Updated Findings

### Fully Verified Benchmarks

| Benchmark | Module Invocations | Est. API Calls | Success |
|-----------|-------------------|----------------|---------|
| invariants_todo | 1 | 1-3 | ✅ 7/7 |
| option_todo | 1 | 1-3 | ✅ 15/15 |
| rwlock_vstd_todo | 1 | 1-3 | ✅ 5/5 |
| transfer_todo | 1 | 1-3 | ✅ 5/5 |
| vectors_todo | 2 | 2-6 | ✅ 16/16 |
| atomics_todo | 3 | 3-9 | ✅ 11/11 |
| bitmap_todo | 5 | 5-15 | ✅ 14/14 |
| rb_type_invariant_todo | 5 | 5-15 | ✅ 13/13 |
| set_from_vec_todo | 5 | 5-15 | ✅ 10/10 |

**Average**: 2.7 module invocations = estimated 4-8 actual API calls

### Module Invocation Counts

| Module | Invocations | Est. API Calls |
|--------|-------------|----------------|
| spec_inference | 57 | 57-171 |
| inv_inference | 30 | 30-90 |
| proof_generation | 29 | 29-87 |
| view_inference | 15 | 15-45 |
| view_refinement | 15 | 15-45 |
| Repair modules | 39 | 39-117 |
| **TOTAL** | **184** | **~200-550** |

---

## Why Actual API Calls Are Unknown

All experiments used **cached responses**:
- No actual API calls were made
- Statistics show: `"total": 0` LLM calls
- All responses came from cache

To get actual counts:
1. Find original uncached logs
2. Run with `ENABLE_LLM_CACHE=0`
3. Count retry attempts in logs

---

## What's Still Valid ✅

These findings remain correct:
- ✅ Module invocation patterns
- ✅ Module sequences (view → inv → spec → proof)
- ✅ spec_inference used in 100% of successes
- ✅ Simple cases need 1 invocation, complex need 5
- ✅ Verification progression per module

---

## Corrected Files

1. **CORRECTED_ANALYSIS.md** - Detailed explanation of the issue
2. **CORRECTION_SUMMARY.txt** - Quick reference correction
3. **FULLY_VERIFIED_BENCHMARKS.md** - Updated with clarifications
4. **README_UPDATED.md** - This file

⚠️ **Other files (CSV, JSON, plots) still use "calls" to mean "invocations"**

---

## Bottom Line

**You were absolutely right to question this!**

- **Module invocations**: 184 (known, exact)
- **Actual API calls**: Unknown, estimated 200-550
- **Each invocation**: Can make 1-3 API calls (1-10 for baseline)

The patterns and insights are still valid, but the scale of actual API usage is 1-3× higher than originally reported.

---

**Updated**: October 16, 2025
**Issue**: Conflated module invocations with API calls
**Status**: ✅ Corrected
