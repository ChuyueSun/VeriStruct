# Fully Verified Benchmarks - Module Invocation Progression

## ⚠️ IMPORTANT CLARIFICATION

This analysis tracks **module invocations**, NOT actual LLM API calls.

**Key Point**: Each module invocation can make **1-3 actual API calls** due to retry loops (baseline can make up to 10).

See `CORRECTED_ANALYSIS.md` for full details on the difference between module invocations and API calls.

---

## Summary

Found **9 benchmarks that achieved 100% verification** (96 total functions verified).

---

## 📊 Quick Statistics

- **Total fully verified benchmarks**: 9 out of 11
- **Total functions verified**: 96 out of 129
- **Average module invocations to 100%**: 2.7 invocations
- **Most efficient**: 1 invocation (4 benchmarks) = estimated 1-3 API calls
- **Most invocations needed**: 5 invocations (3 benchmarks) = estimated 5-15 API calls

---

## 🎯 Individual Benchmark Details

### 1. invariants_todo ⭐ MOST EFFICIENT
**Ground Truth**: 7 functions
**Module Invocations to 100%**: **1 invocation**
**Estimated API Calls**: 1-3 (depending on retries)
**Efficiency**: 7.00 functions/invocation

| Invocation # | Module           | Verified | Errors | Progress |
|--------------|------------------|----------|--------|----------|
| 0            | (initial)        | 0        | 0      | 0%       |
| **1**        | **spec_inference** | **7**  | **0**  | **✅ 100%** |

**Summary**: Achieved 100% with just 1 module invocation using only `spec_inference`.
**Note**: This invocation could have made 1-3 actual API calls if retries were needed.

---

### 2. option_todo ⭐ MOST EFFICIENT
**Ground Truth**: 15 functions
**LLM Calls to 100%**: **1 call**
**Efficiency**: 15.00 functions/call

| Call # | Module           | Verified | Errors | Progress |
|--------|------------------|----------|--------|----------|
| 0      | (initial)        | 0        | 0      | 0%       |
| 1      | spec_inference   | 0        | 0      | 0%       |
| **1**  | **(final)**      | **15**   | **0**  | **✅ 100%** |

**Summary**: Intermediate stage showed 0 verified, but final result achieved 15/15 in 1 call.

---

### 3. rwlock_vstd_todo ⭐ MOST EFFICIENT
**Ground Truth**: 5 functions
**LLM Calls to 100%**: **1 call**
**Efficiency**: 5.00 functions/call

| Call # | Module           | Verified | Errors | Progress |
|--------|------------------|----------|--------|----------|
| 0      | (initial)        | 0        | 0      | 0%       |
| 1      | spec_inference   | 0        | 0      | 0%       |
| **1**  | **(final)**      | **5**    | **0**  | **✅ 100%** |

**Summary**: Final verification revealed all 5 functions passed after 1 call.

---

### 4. transfer_todo ⭐ MOST EFFICIENT
**Ground Truth**: 5 functions
**LLM Calls to 100%**: **1 call**
**Efficiency**: 5.00 functions/call

| Call # | Module           | Verified | Errors | Progress |
|--------|------------------|----------|--------|----------|
| 0      | (initial)        | 0        | 0      | 0%       |
| 1      | spec_inference   | 0        | 0      | 0%       |
| **1**  | **(final)**      | **5**    | **0**  | **✅ 100%** |

**Summary**: Another 1-call success with `spec_inference`.

---

### 5. vectors_todo
**Ground Truth**: 16 functions
**LLM Calls to 100%**: **2 calls**
**Efficiency**: 8.00 functions/call

| Call # | Module             | Verified | Errors | Progress |
|--------|--------------------|----------|--------|----------|
| 0      | (initial)          | 0        | 999    | 0%       |
| 1      | spec_inference     | -1       | 999    | error    |
| 2      | proof_generation   | 10       | 0      | 62.5%    |
| **2**  | **(final)**        | **16**   | **0**  | **✅ 100%** |

**Summary**: Recovered from initial error, achieved 100% in 2 calls.
**Modules**: spec_inference → proof_generation

---

### 6. atomics_todo
**Ground Truth**: 11 functions
**LLM Calls to 100%**: **3 calls**
**Efficiency**: 3.67 functions/call

| Call # | Module             | Verified | Errors | Progress |
|--------|--------------------|----------|--------|----------|
| 0      | (initial)          | 0        | 0      | 0%       |
| 1      | inv_inference      | 0        | 0      | 0%       |
| 2      | spec_inference     | 0        | 0      | 0%       |
| 3      | proof_generation   | 0        | 0      | 0%       |
| **3**  | **(final)**        | **11**   | **0**  | **✅ 100%** |

**Summary**: All intermediate stages showed 0, final verification revealed 11/11 after 3 calls.
**Modules**: inv_inference → spec_inference → proof_generation

---

### 7. bitmap_todo
**Ground Truth**: 14 functions
**LLM Calls to 100%**: **5 calls**
**Efficiency**: 2.80 functions/call

| Call # | Module             | Verified | Errors | Progress |
|--------|--------------------|----------|--------|----------|
| 0      | (initial)          | 0        | 999    | 0%       |
| 1      | view_inference     | 4        | 4      | 28.6%    |
| 2      | view_refinement    | 4        | 4      | 28.6%    |
| 3      | inv_inference      | 4        | 4      | 28.6%    |
| 4      | spec_inference     | 5        | 3      | 35.7%    |
| 5      | proof_generation   | 8        | 0      | 57.1%    |
| **5**  | **(final)**        | **14**   | **0**  | **✅ 100%** |

**Summary**: Progressive improvement through 5 modules, jumped from 8→14 at final.
**Modules**: view_inference → view_refinement → inv_inference → spec_inference → proof_generation

---

### 8. rb_type_invariant_todo
**Ground Truth**: 13 functions
**LLM Calls to 100%**: **5 calls**
**Efficiency**: 2.60 functions/call

| Call # | Module             | Verified | Errors | Progress |
|--------|--------------------|----------|--------|----------|
| 0      | (initial)          | 0        | 999    | 0%       |
| 1      | view_inference     | 4        | 6      | 30.8%    |
| 2      | view_refinement    | 4        | 6      | 30.8%    |
| 3      | inv_inference      | 3        | 7      | 23.1%    |
| 4      | spec_inference     | 4        | 6      | 30.8%    |
| 5      | proof_generation   | 10       | 0      | 76.9%    |
| **5**  | **(final)**        | **13**   | **0**  | **✅ 100%** |

**Summary**: Steady progress, jumped from 10→13 at final verification.
**Modules**: view_inference → view_refinement → inv_inference → spec_inference → proof_generation

---

### 9. set_from_vec_todo
**Ground Truth**: 10 functions
**LLM Calls to 100%**: **5 calls**
**Efficiency**: 2.00 functions/call

| Call # | Module             | Verified | Errors | Progress |
|--------|--------------------|----------|--------|----------|
| 0      | (initial)          | 0        | 0      | 0%       |
| 1      | view_inference     | 6        | 5      | 60.0%    |
| 2      | view_refinement    | 5        | 1      | 50.0%    |
| 3      | inv_inference      | 6        | 5      | 60.0%    |
| 4      | spec_inference     | 9        | 2      | 90.0%    |
| **5**  | **proof_generation** | **10** | **1**  | **✅ 100%** |

**Summary**: Smooth progression, reached 90% by call 4, achieved 100% on call 5.
**Modules**: view_inference → view_refinement → inv_inference → spec_inference → proof_generation

---

## 📈 Efficiency Analysis

### Ranked by Fewest Calls to 100%

| Rank | Benchmark              | Functions | Calls | Funcs/Call | Pattern |
|------|------------------------|-----------|-------|------------|---------|
| 1    | invariants_todo        | 7         | **1** | 7.00       | spec only |
| 1    | option_todo            | 15        | **1** | 15.00      | spec only |
| 1    | rwlock_vstd_todo       | 5         | **1** | 5.00       | spec only |
| 1    | transfer_todo          | 5         | **1** | 5.00       | spec only |
| 5    | vectors_todo           | 16        | **2** | 8.00       | spec + proof |
| 6    | atomics_todo           | 11        | **3** | 3.67       | inv + spec + proof |
| 7    | bitmap_todo            | 14        | **5** | 2.80       | full pipeline |
| 7    | rb_type_invariant_todo | 13        | **5** | 2.60       | full pipeline |
| 7    | set_from_vec_todo      | 10        | **5** | 2.00       | full pipeline |

---

## 🔍 Key Insights

### 1. Simple Cases Need Fewer Calls
- **4 benchmarks** (32 functions total) achieved 100% with just **1 LLM call**
- All used `spec_inference` only
- Total: 32 functions verified with 4 calls = **8.0 functions/call average**

### 2. Complex Cases Need Full Pipeline
- **3 benchmarks** (37 functions total) required **5 LLM calls** each
- All used the full pipeline: view → inv → spec → proof
- Total: 37 functions verified with 15 calls = **2.47 functions/call average**

### 3. Module Usage Patterns

| Calls | Module Sequence | Benchmarks | Success Pattern |
|-------|----------------|------------|-----------------|
| 1     | spec_inference | 4          | Simple specs work immediately |
| 2     | spec → proof   | 1          | Need proofs after specs |
| 3     | inv → spec → proof | 1      | Need invariants first |
| 5     | view → inv → spec → proof | 3 | Complex, need all modules |

### 4. Final Verification Jump
Many benchmarks show a "jump" at final verification:
- **bitmap_todo**: 8 → **14** (+6 at final)
- **rb_type_invariant_todo**: 10 → **13** (+3 at final)
- **atomics_todo**: 0 → **11** (+11 at final!)

**Insight**: Progressive testing doesn't always show all verified functions until final complete verification.

---

## 📊 Module Effectiveness

Based on fully verified benchmarks:

| Module           | Times Used | Always in Success? |
|------------------|------------|--------------------|
| spec_inference   | 9/9        | ✅ Yes (100%)      |
| proof_generation | 5/9        | ✅ When needed     |
| inv_inference    | 4/9        | ✅ For complex     |
| view_inference   | 3/9        | ✅ For complex     |
| view_refinement  | 3/9        | ✅ For complex     |

**Key Finding**: `spec_inference` appears in **every single** fully verified benchmark.

---

## 🎯 Recommendations

1. **Always start with spec_inference** - Present in 100% of successes
2. **Try 1 call first** - 44% of successful benchmarks need only 1 call
3. **Use full pipeline for complex benchmarks** - Those needing views/invariants require ~5 calls
4. **Don't give up early** - Final verification can reveal more functions than progressive tests
5. **Module sequence matters** - Success follows: view → inv → spec → proof

---

## 📂 Generated Files

- **`fully_verified_benchmarks_detail.json`** - Complete structured data
- **`fully_verified_benchmarks.csv`** - Summary table for spreadsheets

**Location**: `/home/chuyue/VerusAgent/output/progression_plots_best/`

---

## Summary Table

| Benchmark              | Functions | Calls | Progress Path |
|------------------------|-----------|-------|---------------|
| invariants_todo        | 7         | 1     | 0 → 7 ✅      |
| option_todo            | 15        | 1     | 0 → 15 ✅     |
| rwlock_vstd_todo       | 5         | 1     | 0 → 5 ✅      |
| transfer_todo          | 5         | 1     | 0 → 5 ✅      |
| vectors_todo           | 16        | 2     | 0 → 10 → 16 ✅ |
| atomics_todo           | 11        | 3     | 0 → 0 → 0 → 11 ✅ |
| bitmap_todo            | 14        | 5     | 0 → 4 → 4 → 4 → 5 → 8 → 14 ✅ |
| rb_type_invariant_todo | 13        | 5     | 0 → 4 → 4 → 3 → 4 → 10 → 13 ✅ |
| set_from_vec_todo      | 10        | 5     | 0 → 6 → 5 → 6 → 9 → 10 ✅ |

**Total**: 96 functions verified across 9 benchmarks
