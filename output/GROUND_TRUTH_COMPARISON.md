# Ground Truth Comparison

**Date**: October 14, 2025

Comparison of experiment results with ground truth benchmarks.

---

## Ground Truth Benchmark Verification

| Benchmark | Ground Truth File | Verified Functions | Status |
|-----------|------------------|-------------------|--------|
| atomics_todo | atomics.rs | 11 | ✅ |
| bitmap_todo | bitmap_1.rs | 15 | ✅ |
| bst_map_todo | treemap.rs | 21 | ✅ |
| invariants_todo | invariants.rs | 7 | ✅ |
| node_todo | node.rs | 12 | ✅ |
| option_todo | option.rs | 15 | ✅ |
| rb_type_invariant_todo | rb_type_invariant.rs | 13 | ✅ |
| rwlock_vstd_todo | rwlock_vstd.rs | 5 | ✅ |
| set_from_vec_todo | set_from_vec.rs | 10 | ✅ |
| transfer_todo | transfer.rs | 5 | ✅ |
| vectors_todo | vectors.rs | 16 | ✅ |

---

## Fully Verified Benchmarks by Experiment

| Benchmark | 20251011_135504 | 20251011_160501 | 20251010_152504 | 20251010_215225 | 20251011_160518 | 20251010_193940 | 20251011_182336 | Ground Truth |
|-----------|------|------|------|------|------|------|------|-------------|
| **atomics_todo** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | 11 |
| **bitmap_todo** | - | - | - | - | - | - | - | 15 |
| **bst_map_todo** | - | - | - | - | - | - | - | 21 |
| **invariants_todo** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 7 |
| **node_todo** | - | - | - | - | - | - | - | 12 |
| **option_todo** | ✅ | ✅ | ✅ | ✅ | ✅ | - | ✅ | 15 |
| **rb_type_invariant_todo** | ✅ | ✅ | ✅ | - | ✅ | ✅ | - | 13 |
| **rwlock_vstd_todo** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 5 |
| **set_from_vec_todo** | - | - | - | - | ✅ | - | - | 10 |
| **transfer_todo** | ✅ | - | ✅ | ✅ | ✅ | ✅ | ✅ | 5 |
| **vectors_todo** | - | - | - | - | - | - | - | 16 |
| **FULL COUNT** | **6/11** | **5/11** | **6/11** | **5/11** | **7/11** | **5/11** | **4/11** | - |

---

### repair_experiment_20251011_135504

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 7 | 46.7% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 13 | 100.0% | ✅ FULL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 11 | 110.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 7 | 43.8% | 🔶 PARTIAL |
| **TOTAL** | **130** | **91** | **70.0%** | |

### repair_experiment_20251011_160501

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 4 | 26.7% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 13 | 100.0% | ✅ FULL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 11 | 110.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 4 | 80.0% | 🔶 PARTIAL |
| vectors_todo | 16 | 7 | 43.8% | 🔶 PARTIAL |
| **TOTAL** | **130** | **87** | **66.9%** | |

### repair_experiment_20251010_152504

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 4 | 26.7% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 13 | 100.0% | ✅ FULL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 11 | 110.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 7 | 43.8% | 🔶 PARTIAL |
| **TOTAL** | **130** | **88** | **67.7%** | |

### repair_experiment_20251010_215225

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 8 | 53.3% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 0 | 0.0% | ❌ NONE |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 11 | 110.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 6 | 37.5% | 🔶 PARTIAL |
| **TOTAL** | **130** | **78** | **60.0%** | |

### repair_experiment_20251011_160518

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 4 | 26.7% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 13 | 100.0% | ✅ FULL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 10 | 100.0% | ✅ FULL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 7 | 43.8% | 🔶 PARTIAL |
| **TOTAL** | **130** | **87** | **66.9%** | |

### repair_experiment_20251010_193940

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 11 | 100.0% | ✅ FULL |
| bitmap_todo | 15 | 8 | 53.3% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 0 | 0.0% | ❌ NONE |
| rb_type_invariant_todo | 13 | 13 | 100.0% | ✅ FULL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 6 | 60.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 6 | 37.5% | 🔶 PARTIAL |
| **TOTAL** | **130** | **71** | **54.6%** | |

### repair_experiment_20251011_182336

| Benchmark | Ground Truth | Experiment | Completion | Status |
|-----------|--------------|------------|------------|--------|
| atomics_todo | 11 | 0 | 0.0% | ❌ NONE |
| bitmap_todo | 15 | 7 | 46.7% | 🔶 PARTIAL |
| bst_map_todo | 21 | 10 | 47.6% | 🔶 PARTIAL |
| invariants_todo | 7 | 7 | 100.0% | ✅ FULL |
| node_todo | 12 | 0 | 0.0% | ❌ NONE |
| option_todo | 15 | 15 | 100.0% | ✅ FULL |
| rb_type_invariant_todo | 13 | 9 | 69.2% | 🔶 PARTIAL |
| rwlock_vstd_todo | 5 | 5 | 100.0% | ✅ FULL |
| set_from_vec_todo | 10 | 11 | 110.0% | 🔶 PARTIAL |
| transfer_todo | 5 | 5 | 100.0% | ✅ FULL |
| vectors_todo | 16 | 0 | 0.0% | ❌ NONE |
| **TOTAL** | **130** | **69** | **53.1%** | |
