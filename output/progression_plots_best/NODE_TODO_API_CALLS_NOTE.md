# Node Todo - API Calls Progression Note

**Date**: October 16, 2025
**Source**: node_test_v3 run (20251011_160727)

## Entry Added to exact_api_calls_to_verified.csv

```csv
node_todo,12,8,8,10,11,-,-,-,11,0*,91.7%
```

## Explanation

### Ground Truth: 12
The original `node.rs` file (with test_node1 and test_node2 added) verifies **12 functions** with **0 errors**.

### Progression Data

Unlike other benchmarks in the CSV that track actual LLM API calls, the node_test_v3 run shows **0 tracked API calls** in its statistics. Instead, we have **stage-based progression**:

| Stage | Verified | Errors | Notes |
|-------|----------|--------|-------|
| **Preprocessed** (API_Call_0) | 8 | 4 | Starting state after preprocessing |
| **View/Inv Inference** (API_Call_1) | 8 | 4 | No improvement through these stages |
| **Spec Inference** (API_Call_2) | 10 | 2 | ✅ +2 verified functions |
| **Repair (postcond)** (API_Call_3) | 11 | 1 | ✅ +1 verified function |
| **Final** | 11 | 1 | Final state with 1 remaining error |

### Key Points

1. **Total_API_Calls: 0\***
   - Marked with asterisk (\*) to indicate this is stage-based progression
   - The statistics show `total_llm_calls: 0`
   - Progression tracked through pipeline stages, not individual API calls

2. **Success Rate: 91.7%**
   - Achieved: 11 out of 12 functions verified
   - 1 remaining error in `delete_from_optional` function
   - Better than any previous repair_experiments run (which had 3 errors)

3. **Starting Point Different**
   - Most other benchmarks start with 0 verified at API_Call_0
   - node_todo starts with 8 verified at preprocessed stage
   - This is because the preprocessed code already had some working verification

## Comparison with Other Benchmarks

### Similar Benchmarks (by ground truth size)
- **set_from_vec_todo**: 10 ground truth, 10 verified, 6 API calls → 100%
- **atomics_todo**: 11 ground truth, 11 verified, 4 API calls → 100%
- **node_todo**: 12 ground truth, 11 verified, 0* stages → 91.7%

### Why Only 91.7%?

The remaining 1 error is a complex assertion in `delete_from_optional`:
```rust
assert(Node::<V>::optional_as_map(*node) =~=
       Node::<V>::optional_as_map(*old(node)).remove(key));
```

This assertion about map equivalence after deletion requires deeper reasoning about BST and map operation properties that the repair mechanisms couldn't automatically resolve.

## Data Sources

- **Run**: `/home/chuyue/VerusAgent/output/node_test_v3/node_todo/20251011_160727/`
- **Statistics**: `statistics/summary_node_todo_20251011_162211.json`
- **Detailed Statistics**: `statistics/detailed_node_todo_20251011_162211.json`
- **Progression Report**: `VERIFICATION_PROGRESSION_REPORT.md`
- **Verification Stats**: `verification_progression_stats.json`

## Note on Methodology

This entry uses a different tracking methodology than other benchmarks:
- **Other benchmarks**: Track individual LLM API calls during generation
- **node_todo**: Track pipeline stages (4 modules activated: view_inference, view_refinement, inv_inference, spec_inference)

The 0* notation in Total_API_Calls indicates this hybrid approach where stages are used instead of API call counts.
