# Exact LLM API Calls for Fully Verified Benchmarks

## Summary

Found **EXACT API call counts** by analyzing saved prompt files (each prompt file = 1 actual LLM API call).

---

## Fully Verified Benchmarks - Exact API Calls

| Benchmark | Ground Truth | Total API Calls | Breakdown by Module | Result |
|-----------|--------------|-----------------|---------------------|---------|
| invariants_todo | 7 | **3** | spec: 2, other: 1 | ✅ 7/7 |
| rwlock_vstd_todo | 5 | **3** | spec: 2, other: 1 | ✅ 5/5 |
| option_todo | 15 | **6** | spec: 1, repair_syntax: 4, other: 1 | ✅ 15/15 |
| transfer_todo | 5 | **10** | spec: 10 (baseline) | ✅ 5/5 |
| atomics_todo | 11 | **12** | spec: 3, repair_syntax: 8, other: 1 | ✅ 11/11 |
| vectors_todo | 16 | **17** | spec: 4, repair_syntax: 12, other: 1 | ✅ 16/16 |
| bitmap_todo | 14 | **18** | spec: 13, proof: 2, repair_syntax: 2, other: 1 | ✅ 14/14 |
| rb_type_invariant_todo | 13 | **33** | spec: 10, proof: 4, repair_syntax: 18, other: 1 | ✅ 13/13 |
| set_from_vec_todo | 10 | **36** | spec: 5, repair_syntax: 30, other: 1 | ✅ 10/10 |

**Total**: 96 functions verified with **138 actual LLM API calls**

---

## Detailed Breakdown by Benchmark

### 1. invariants_todo (MOST EFFICIENT)
**Ground Truth**: 7 functions
**Total API Calls**: **3 calls**
**Efficiency**: 2.33 functions per call

**Module Breakdown**:
- spec_inference: 2 calls
- other: 1 call

**Experiment**: `repair_experiment_20251011_135504/no_repairs/invariants_todo`

✅ **Result**: 7/7 verified (100%)

---

### 2. rwlock_vstd_todo (MOST EFFICIENT)
**Ground Truth**: 5 functions
**Total API Calls**: **3 calls**
**Efficiency**: 1.67 functions per call

**Module Breakdown**:
- spec_inference: 2 calls
- other: 1 call

**Experiment**: `repair_experiment_20251011_135504/no_repairs/rwlock_vstd_todo`

✅ **Result**: 5/5 verified (100%)

---

### 3. option_todo
**Ground Truth**: 15 functions
**Total API Calls**: **6 calls**
**Efficiency**: 2.50 functions per call

**Module Breakdown**:
- repair_syntax: 4 calls
- spec_inference: 1 call
- other: 1 call

**Experiment**: `repair_experiment_20251010_193940/full_pipeline/option_todo`

✅ **Result**: 15/15 verified (100%)

---

### 4. transfer_todo
**Ground Truth**: 5 functions
**Total API Calls**: **10 calls**
**Efficiency**: 0.50 functions per call

**Module Breakdown**:
- spec_inference: 10 calls (baseline sampling strategy)

**Experiment**: `repair_experiment_20251011_135504/baseline/transfer_todo`

**Note**: This used the baseline approach which generates multiple samples per call.

✅ **Result**: 5/5 verified (100%)

---

### 5. atomics_todo
**Ground Truth**: 11 functions
**Total API Calls**: **12 calls**
**Efficiency**: 0.92 functions per call

**Module Breakdown**:
- repair_syntax: 8 calls
- spec_inference: 3 calls
- other: 1 call

**Experiment**: `repair_experiment_20251010_215225/full_pipeline/atomics_todo`

✅ **Result**: 11/11 verified (100%)

---

### 6. vectors_todo
**Ground Truth**: 16 functions
**Total API Calls**: **17 calls**
**Efficiency**: 0.94 functions per call

**Module Breakdown**:
- repair_syntax: 12 calls
- spec_inference: 4 calls
- other: 1 call

**Experiment**: `repair_experiment_20251010_193940/full_pipeline/vectors_todo`

✅ **Result**: 16/16 verified (100%)

---

### 7. bitmap_todo
**Ground Truth**: 14 functions
**Total API Calls**: **18 calls**
**Efficiency**: 0.78 functions per call

**Module Breakdown**:
- spec_inference: 13 calls
- proof_generation: 2 calls
- repair_syntax: 2 calls
- other: 1 call

**Experiment**: `repair_experiment_20251010_191429/full_pipeline/bitmap_todo`

✅ **Result**: 14/14 verified (100%)

---

### 8. rb_type_invariant_todo
**Ground Truth**: 13 functions
**Total API Calls**: **33 calls**
**Efficiency**: 0.39 functions per call

**Module Breakdown**:
- repair_syntax: 18 calls
- spec_inference: 10 calls
- proof_generation: 4 calls
- other: 1 call

**Experiment**: `repair_experiment_20251011_225133/full_pipeline/rb_type_invariant_todo`

✅ **Result**: 13/13 verified (100%)

---

### 9. set_from_vec_todo (MOST CALLS)
**Ground Truth**: 10 functions
**Total API Calls**: **36 calls**
**Efficiency**: 0.28 functions per call

**Module Breakdown**:
- repair_syntax: 30 calls (!!)
- spec_inference: 5 calls
- other: 1 call

**Experiment**: `repair_experiment_20251011_225137/full_pipeline/set_from_vec_todo`

**Note**: Needed extensive syntax repairs (30 calls).

✅ **Result**: 10/10 verified (100%)

---

## Key Statistics

### Overall:
- **Total Functions Verified**: 96/96 (100%)
- **Total API Calls**: 138 calls
- **Average Efficiency**: 0.70 functions per call
- **Best Efficiency**: 2.33 funcs/call (invariants_todo)
- **Worst Efficiency**: 0.28 funcs/call (set_from_vec_todo)

### API Calls by Module Type:

| Module | Total Calls | % of Total |
|--------|-------------|------------|
| spec_inference | 50 | 36.2% |
| repair_syntax | 74 | 53.6% |
| proof_generation | 6 | 4.3% |
| other | 8 | 5.8% |

**Surprising Finding**: repair_syntax dominated (54% of calls)!

### Distribution:
- **Fewest calls**: 3 calls (invariants_todo, rwlock_vstd_todo)
- **Most calls**: 36 calls (set_from_vec_todo)
- **Median**: 12 calls
- **Mean**: 15.3 calls per benchmark

---

## Comparison: Module Invocations vs Actual API Calls

| Benchmark | Module Invocations | Actual API Calls | Ratio |
|-----------|-------------------|------------------|-------|
| invariants_todo | 1 | 3 | 3.0× |
| rwlock_vstd_todo | 1 | 3 | 3.0× |
| option_todo | 1 | 6 | 6.0× |
| transfer_todo | 1 | 10 | 10.0× |
| atomics_todo | 3 | 12 | 4.0× |
| vectors_todo | 2 | 17 | 8.5× |
| bitmap_todo | 5 | 18 | 3.6× |
| rb_type_invariant_todo | 5 | 33 | 6.6× |
| set_from_vec_todo | 5 | 36 | 7.2× |

**Average Ratio**: **5.8× more API calls than module invocations**

This is because:
1. Each module can retry 1-3 times (retry loop)
2. Safety checks may fail, triggering retries
3. Different error conditions require multiple attempts

---

## Key Insights

### 1. Repair Dominates
**repair_syntax made 74 calls (54%)** - much more than expected!
- set_from_vec_todo needed 30 repair calls
- rb_type_invariant_todo needed 18 repair calls
- vectors_todo needed 12 repair calls

### 2. Simple Cases Are Very Efficient
- invariants_todo: 3 calls → 7 functions (2.33 funcs/call)
- rwlock_vstd_todo: 3 calls → 5 functions (1.67 funcs/call)

### 3. Complex Cases Need Many Attempts
- set_from_vec_todo: 36 calls for 10 functions
- rb_type_invariant_todo: 33 calls for 13 functions
- Multiple repair attempts were needed

### 4. Baseline Can Be Expensive
- transfer_todo: 10 calls (baseline sampling)
- Baseline generates many samples per call

### 5. Actual Ratio is ~6× Higher Than Invocations
- Module invocations: 2.7 average
- Actual API calls: 15.3 average
- Ratio: 5.7× more calls

---

## Recommendations

Based on these exact numbers:

1. **Budget for repairs**: ~50% of calls may be repairs
2. **Simple specs are cheap**: 3-6 calls for simple cases
3. **Complex repairs are expensive**: Up to 30 calls for syntax issues
4. **Average budget**: ~15 API calls per benchmark
5. **Monitor repair loops**: They can dominate cost

---

## Data Source

- **Method**: Counted saved prompt files (1 file = 1 API call)
- **Location**: `{experiment_dir}/prompts/*.md`
- **Files analyzed**: All experiments in `repair_experiments/`
- **Total benchmarks**: 9 fully verified
- **Data file**: `exact_llm_calls.json`

---

**Analysis Date**: October 16, 2025
**Total API Calls Counted**: 138
**Method**: Prompt file analysis (exact count)
