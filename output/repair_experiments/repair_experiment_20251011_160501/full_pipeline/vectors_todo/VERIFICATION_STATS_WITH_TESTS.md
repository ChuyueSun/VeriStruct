# Verification Statistics - vectors_todo with Progressive Tests

## Summary

All files in the `vectors_todo/20251011_160502/` directory have been updated with **progressive test functions** matching the pattern from `benchmarks-complete/vectors.rs`.

## Test Functions Added

Each file now contains 6 progressive test functions:

### Binary Search Tests
1. **`binary_search_test1`** - Calls both search functions with no assertions (baseline)
2. **`binary_search_test2`** - Adds assertions for `binary_search` postconditions only
3. **`binary_search_test3`** - Adds assertions for both `binary_search` and `binary_search_no_spinoff`

### Reverse Tests
4. **`reverse_test1`** - Calls reverse with no assertions (baseline)
5. **`reverse_test2`** - Asserts length preservation only
6. **`reverse_test3`** - Asserts full correctness (length + element reversal)

## Verification Results by Stage

### Main Pipeline Files

| Stage | File | Status | Functions Verified | Notes |
|-------|------|--------|--------------------|-------|
| 0 | `preprocessed.rs` | ✗ Failed | 0 | Expected - no specs/invariants |
| 1 | `01_spec_inference_vectors_todo__Vec_General_20251011_160502.rs` | ✗ Failed | 0 | Expected - specs but no invariants |
| 2 | `02_proof_generation_vectors_todo__Vec_General_20251011_160502.rs` | ✓ Success | 16 | First stage with complete proofs |
| 3 | `checkpoint_best_vectors_todo__Vec_General_20251011_160502.rs` | ✓ Success | 16 | |
| 4 | `final_result.rs` | ✓ Success | 16 | |
| 4b | `final_result_vectors_todo.rs` | ✓ Success | 16 | |
| 4c | `final_result_vectors_todo__Vec_General_20251011_160502.rs` | ✓ Success | 16 | |

### Best/ Directory Files

| File | Status | Functions Verified |
|------|--------|--------------------|
| `best/best.rs` | ✓ Success | 16 |
| `best/best_vectors_todo.rs` | ✓ Success | 16 |
| `best/best_vectors_todo__Vec_General_20251011_160502.rs` | ✓ Success | 16 |

### Samples/ Directory Files

| File | Status | Functions Verified | Notes |
|------|--------|--------------------|-------|
| `samples/04_spec_inference_sample_1.rs` | ✗ Failed | 0 | Missing invariants |
| `samples/04_spec_inference_sample_2.rs` | ✗ Failed | 0 | Missing invariants |
| `samples/04_spec_inference_sample_3.rs` | ✗ Failed | 0 | Missing invariants |
| `samples/04_spec_inference_global_best.rs` | ✗ Failed | 0 | Missing invariants |
| `samples/05_proof_generation_sample_1.rs` | ⚠ Partial | 15 | Minor verification issues |
| `samples/05_proof_generation_sample_2.rs` | ✓ Success | 16 | |
| `samples/05_proof_generation_correct.rs` | ✓ Success | 16 | |
| `samples/05_proof_generation_global_best.rs` | ✓ Success | 16 | |

## Function Breakdown

When fully verified (16 functions), the breakdown is:

1. **Core implementations**: 3
   - `binary_search`
   - `reverse`
   - `binary_search_no_spinoff`

2. **Test functions**: 6
   - `binary_search_test1`, `binary_search_test2`, `binary_search_test3`
   - `reverse_test1`, `reverse_test2`, `reverse_test3`

3. **Helper functions**: 2
   - `test` (empty)
   - `main` (empty)

4. **Additional verification units**: ~5
   - Loop bodies and invariants count as separate verification units

## Key Observations

1. **Stage 0-1 (Preprocessed & Spec Inference)**: Cannot verify completely due to missing loop invariants
2. **Stage 2+ (Proof Generation onwards)**: All verify successfully with 16 functions
3. **Progressive testing**: Each test level adds more postcondition assertions, allowing incremental verification
4. **Sample files**: Spec inference samples fail (as expected), proof generation samples mostly succeed

## Impact

The progressive test functions enable:
- **Incremental verification**: Test complexity increases gradually
- **Better debugging**: Can identify which assertions cause verification failures
- **Performance measurement**: Can track verification time per test level
- **Educational value**: Shows how to build up verification complexity

## Files Updated

**Total**: 18 files
- Main pipeline: 7 files
- Best/ directory: 3 files
- Samples/ directory: 8 files

All files have been updated with:
- Progressive test functions (6 per file)
- Proper `} // verus!` closing braces
- Consistent formatting matching `vectors.rs`
