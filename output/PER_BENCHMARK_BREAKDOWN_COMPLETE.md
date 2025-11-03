# Per-Benchmark Breakdown of Verified Functions

**Date**: October 14, 2025

Analysis of the top 7 repair experiments showing verified functions per benchmark.

## Two Counting Methodologies

This report provides two perspectives:

1. **SUM Method** (Original): Sums all verification results from all files (final_result.rs + best.rs)
   - Matches the original `count_by_experiment.py` methodology
   - May count some functions multiple times if they appear in both files

2. **MAX Method** (Alternative): Takes the maximum verified functions per benchmark
   - Avoids potential double-counting
   - More conservative estimate

---

## Original Methodology: Sum All Files

*This matches the counts in EXPERIMENTS_RANKED_BY_FUNCTIONS.md*

### Summary Table

| Benchmark | 20251011_135504 | 20251011_160501 | 20251010_152504 | 20251010_215225 | 20251011_160518 | 20251010_193940 | 20251011_182336 |
|-----------|------------|------------|------------|------------|------------|------------|------------|
| **atomics_todo** | 11 | 11 | 11 | 11 | 11 | 11 | 0 |
| **bitmap_todo** | 14 | 8 | 4 | 16 | 4 | 16 | 7 |
| **bst_map_todo** | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| **invariants_todo** | 14 | 14 | 14 | 14 | 14 | 14 | 14 |
| **option_todo** | 15 | 15 | 15 | 15 | 15 | 0 | 15 |
| **rb_type_invariant_todo** | 13 | 13 | 13 | 0 | 13 | 13 | 9 |
| **rwlock_vstd_todo** | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| **set_from_vec_todo** | 22 | 22 | 22 | 22 | 10 | 12 | 22 |
| **transfer_todo** | 5 | 4 | 5 | 5 | 5 | 5 | 5 |
| **vectors_todo** | 7 | 7 | 7 | 6 | 7 | 6 | 0 |
| **TOTAL** | **116** | **109** | **106** | **104** | **94** | **92** | **87** |

## Alternative: Max Per Benchmark

*Conservative count avoiding potential duplicates*

### Summary Table

| Benchmark | 20251011_135504 | 20251011_160501 | 20251010_152504 | 20251010_215225 | 20251011_160518 | 20251010_193940 | 20251011_182336 |
|-----------|------------|------------|------------|------------|------------|------------|------------|
| **atomics_todo** | 11 | 11 | 11 | 11 | 11 | 11 | 0 |
| **bitmap_todo** | 7 | 4 | 4 | 8 | 4 | 8 | 7 |
| **bst_map_todo** | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| **invariants_todo** | 7 | 7 | 7 | 7 | 7 | 7 | 7 |
| **option_todo** | 15 | 15 | 15 | 15 | 15 | 0 | 15 |
| **rb_type_invariant_todo** | 13 | 13 | 13 | 0 | 13 | 13 | 9 |
| **rwlock_vstd_todo** | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| **set_from_vec_todo** | 11 | 11 | 11 | 11 | 10 | 6 | 11 |
| **transfer_todo** | 5 | 4 | 5 | 5 | 5 | 5 | 5 |
| **vectors_todo** | 7 | 7 | 7 | 6 | 7 | 6 | 0 |
| **TOTAL** | **91** | **87** | **88** | **78** | **87** | **71** | **69** |

---

## Detailed Breakdown (SUM Method)

### repair_experiment_20251011_135504

**Total**: 116 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 0 + 11 |
| bitmap_todo | 14 | 2 | 7 + 7 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 0 + 15 |
| rb_type_invariant_todo | 13 | 2 | 0 + 13 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 22 | 2 | 11 + 11 |
| transfer_todo | 5 | 2 | 5 + 0 |
| vectors_todo | 7 | 2 | 0 + 7 |

### repair_experiment_20251011_160501

**Total**: 109 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 11 + 0 |
| bitmap_todo | 8 | 2 | 4 + 4 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 15 + 0 |
| rb_type_invariant_todo | 13 | 2 | 0 + 13 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 22 | 2 | 11 + 11 |
| transfer_todo | 4 | 2 | 0 + 4 |
| vectors_todo | 7 | 2 | 0 + 7 |

### repair_experiment_20251010_152504

**Total**: 106 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 11 + 0 |
| bitmap_todo | 4 | 1 | 4 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 15 + 0 |
| rb_type_invariant_todo | 13 | 2 | 0 + 13 |
| rwlock_vstd_todo | 5 | 2 | 5 + 0 |
| set_from_vec_todo | 22 | 2 | 11 + 11 |
| transfer_todo | 5 | 2 | 5 + 0 |
| vectors_todo | 7 | 2 | 7 + 0 |

### repair_experiment_20251010_215225

**Total**: 104 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 11 + 0 |
| bitmap_todo | 16 | 2 | 8 + 8 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 0 + 15 |
| rb_type_invariant_todo | 0 | 1 | 0 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 22 | 2 | 11 + 11 |
| transfer_todo | 5 | 2 | 5 + 0 |
| vectors_todo | 6 | 2 | 0 + 6 |

### repair_experiment_20251011_160518

**Total**: 94 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 11 + 0 |
| bitmap_todo | 4 | 1 | 4 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 15 + 0 |
| rb_type_invariant_todo | 13 | 2 | 0 + 13 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 10 | 1 | 10 |
| transfer_todo | 5 | 2 | 5 + 0 |
| vectors_todo | 7 | 2 | 0 + 7 |

### repair_experiment_20251010_193940

**Total**: 92 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 11 | 2 | 11 + 0 |
| bitmap_todo | 16 | 2 | 8 + 8 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 0 | 2 | 0 + 0 |
| rb_type_invariant_todo | 13 | 2 | 13 + 0 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 12 | 2 | 6 + 6 |
| transfer_todo | 5 | 2 | 0 + 5 |
| vectors_todo | 6 | 2 | 0 + 6 |

### repair_experiment_20251011_182336

**Total**: 87 verified functions across 10 benchmarks

| Benchmark | Verified (SUM) | Files Checked | Per-File Breakdown |
|-----------|----------------|--------------|-------------------|
| atomics_todo | 0 | 2 | 0 + 0 |
| bitmap_todo | 7 | 2 | 0 + 7 |
| bst_map_todo | 10 | 1 | 10 |
| invariants_todo | 14 | 2 | 7 + 7 |
| option_todo | 15 | 2 | 15 + 0 |
| rb_type_invariant_todo | 9 | 2 | 0 + 9 |
| rwlock_vstd_todo | 5 | 2 | 0 + 5 |
| set_from_vec_todo | 22 | 2 | 11 + 11 |
| transfer_todo | 5 | 2 | 0 + 5 |
| vectors_todo | 0 | 2 | 0 + 0 |

---

## Key Observations

### Best Performers by Benchmark

| Benchmark | Best Experiment | Verified Functions |
|-----------|----------------|-------------------|
| atomics_todo | 20251011_135504 | 11 |
| bitmap_todo | 20251010_215225 | 16 |
| bst_map_todo | 20251011_135504 | 10 |
| invariants_todo | 20251011_135504 | 14 |
| option_todo | 20251011_135504 | 15 |
| rb_type_invariant_todo | 20251011_135504 | 13 |
| rwlock_vstd_todo | 20251011_135504 | 5 |
| set_from_vec_todo | 20251011_135504 | 22 |
| transfer_todo | 20251011_135504 | 5 |
| vectors_todo | 20251011_135504 | 7 |
