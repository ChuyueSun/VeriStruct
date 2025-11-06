# Planning System Analysis & Recommendations

## Current Planning System

The planner uses LLM-based workflow selection with **4 predefined workflows:**

### Current Workflows
1. **Full Sequence:** `view_inference → view_refinement → [inv_inference] → spec_inference [→ proof_generation]`
2. **Invariant-First:** `inv_inference → spec_inference [→ proof_generation]`
3. **Specification-Only:** `spec_inference [→ proof_generation]`
4. **Invariant-Only:** `inv_inference [→ proof_generation]`

---

## Problems with Current System

### 1. **Missing Workflow Patterns**

Current workflows don't cover these benchmark needs:

❌ **View without Refinement:**
```
Needed: view_inference → spec_inference → proof_generation
Example: bitmap_2_todo.rs (simple spec fn view)
Current: Forces Full Sequence (includes unnecessary view_refinement)
```

❌ **View with Invariants but no Refinement:**
```
Needed: view_inference → inv_inference → spec_inference → proof_generation
Example: bst_map_todo.rs
Current: Full Sequence includes unnecessary view_refinement
```

❌ **Functions-Only with Proofs:**
```
Needed: spec_inference → proof_generation
Example: vectors_todo.rs (no struct, just functions)
Current: Specification-Only works, but criteria unclear
```

### 2. **view_refinement is Almost Never Needed**

Looking at all benchmarks, **view_refinement is rarely/never actually needed**:
- Most View functions are straightforward mappings
- bitmap_2_todo: Simple Seq<bool> mapping
- bst_map_todo: Simple Map delegation
- rb_type_invariant: Tuple (Seq<T>, usize)

**Recommendation:** Make view_refinement OPTIONAL or remove it entirely from default workflows.

### 3. **Selection Criteria Too Vague**

Current criteria:
- "Code explicitly contains 'View' keyword" → Full Sequence
- But this doesn't distinguish between:
  - Simple `spec fn view` (doesn't need refinement)
  - Complex `impl View for` (might need refinement)
  - Partial `impl View for` with TODO in view function

---

## Recommended New Workflows

### Updated Workflow Set (8 workflows)

| # | Workflow | Modules | Use Case | Example |
|---|----------|---------|----------|---------|
| 1 | **Functions-Only** | `spec_inference → proof_generation` | Standalone functions, no structs | vectors_todo.rs |
| 2 | **Specs-Only** | `spec_inference` | Trait impls, enums | invariants_todo.rs, option_todo.rs |
| 3 | **Simple View** | `view_inference → spec_inference → proof_generation` | spec fn view, no invariants | bitmap_2_todo.rs |
| 4 | **View + Invariants** | `view_inference → inv_inference → spec_inference → proof_generation` | Struct with view and invariants | bst_map_todo.rs |
| 5 | **Complex View** | `view_inference → view_refinement → spec_inference → proof_generation` | Complex view needing refinement | (rarely needed) |
| 6 | **Full Sequence** | `view_inference → view_refinement → inv_inference → spec_inference → proof_generation` | Complex struct with everything | rb_type_invariant_todo.rs |
| 7 | **Invariant-First** | `inv_inference → spec_inference → proof_generation` | Struct with invariants, no view | atomics_todo.rs, node_todo.rs |
| 8 | **Invariant-Only** | `inv_inference` | Just invariants needed | (edge case) |

### Key Changes from Current System

1. ✅ Add **Simple View workflow (#3)** - most common View case
2. ✅ Add **View + Invariants workflow (#4)** - common for data structures
3. ✅ Make **view_refinement OPTIONAL** - only for truly complex cases
4. ✅ Add **proof_generation conditionally** - only when proofs/loops present
5. ✅ Keep **Invariant-First (#7)** - for structs without views

---

## Improved Selection Criteria

### Step 1: Detect Code Structure

```python
has_struct = bool(re.search(r'\bstruct\s+\w+', code))
has_enum = bool(re.search(r'\benum\s+\w+', code))
has_trait_impl = bool(re.search(r'\bimpl\s+\w+.*\bfor\s+\w+', code))
has_functions = bool(re.search(r'\bfn\s+\w+', code))
```

### Step 2: Detect View Requirements

```python
has_spec_fn_view = bool(re.search(r'\bspec\s+fn\s+view\s*\(', code))
has_view_trait = bool(re.search(r'\bimpl.*View\s+for', code))
has_view = has_spec_fn_view or has_view_trait
```

### Step 3: Detect Other Features

```python
has_type_invariant = bool(re.search(r'#\[verifier::type_invariant\]|spec fn.*well_formed', code))
has_proof_todos = 'TODO: add proof' in code or 'TODO: add invariant' in code
has_loop = 'while' in code or 'for' in code
```

### Step 4: Select Workflow

```python
def select_workflow(code):
    workflow = []

    # View handling
    if has_view:
        workflow.append('view_inference')
        # Only add refinement for truly complex cases
        if is_complex_view(code):  # Multiple aspects, nested structures
            workflow.append('view_refinement')

    # Invariants
    if has_struct and has_type_invariant:
        workflow.append('inv_inference')

    # Always need specs if we have functions/methods with TODOs
    if has_functions or has_struct:
        workflow.append('spec_inference')

    # Proofs
    if has_proof_todos or has_loop:
        workflow.append('proof_generation')

    return workflow
```

### Helper: is_complex_view

```python
def is_complex_view(code):
    """Determine if view needs refinement."""
    # Check for tuple views (multiple aspects)
    if 'type V = (' in code:  # Tuple view type
        return True

    # Check for complex nested structures
    if 'Map<' in code and 'Seq<' in code:  # Mixed types
        return True

    # Simple mappings don't need refinement
    if re.search(r'type V = (Seq<|Map<|Set<)\w+>', code):
        return False

    return False
```

---

## Implementation Options

### Option 1: Enhance LLM-Based Planning (Current)

**Pros:**
- Flexible, can handle new patterns
- Already implemented

**Cons:**
- LLM might make mistakes
- Extra LLM call cost/time
- Need careful prompt engineering

**Changes Needed:**
- Update `prompts/plan_system.md` with new workflows
- Add better selection criteria
- Add `is_complex_view` detection logic

### Option 2: Rule-Based Planning (Recommended)

**Pros:**
- ✅ Fast, deterministic, no LLM call
- ✅ Predictable behavior
- ✅ Easy to debug
- ✅ Lower cost

**Cons:**
- Less flexible for edge cases
- Need to maintain rules

**Implementation:**
```python
class RuleBasedPlanner:
    def select_workflow(self, code: str) -> List[str]:
        # Use the detection logic above
        workflow = []

        # Analyze code structure
        has_view = self.detect_view(code)
        has_invariants = self.detect_invariants(code)
        has_proofs = self.detect_proofs(code)
        is_complex = self.is_complex_view(code)

        # Build workflow
        if has_view:
            workflow.append('view_inference')
            if is_complex:
                workflow.append('view_refinement')

        if has_invariants:
            workflow.append('inv_inference')

        workflow.append('spec_inference')

        if has_proofs:
            workflow.append('proof_generation')

        return workflow
```

### Option 3: Hybrid Approach (Best of Both)

**Combine rule-based + LLM validation:**
```python
def select_workflow(code: str) -> List[str]:
    # 1. Rule-based initial selection
    rule_based_workflow = rule_based_planner.select(code)

    # 2. Log the decision
    logger.info(f"Rule-based workflow: {rule_based_workflow}")

    # 3. Optional: Ask LLM to validate/adjust (can skip to save cost)
    # llm_workflow = llm_planner.validate(code, rule_based_workflow)

    return rule_based_workflow
```

---

## Specific Benchmark Workflows

Applying the recommended approach:

```
transfer_todo.rs:         spec_inference → proof_generation
invariants_todo.rs:       spec_inference
rwlock_vstd_todo.rs:      spec_inference
option_todo.rs:           spec_inference
vectors_todo.rs:          spec_inference → proof_generation

atomics_todo.rs:          inv_inference → spec_inference → proof_generation
node_todo.rs:             inv_inference → spec_inference → proof_generation

bitmap_2_todo.rs:         view_inference → spec_inference → proof_generation
bitmap_todo.rs:           view_inference → spec_inference → proof_generation
set_from_vec_todo.rs:     view_inference → spec_inference → proof_generation

bst_map_todo.rs:          view_inference → inv_inference → spec_inference → proof_generation
treemap_todo.rs:          view_inference → inv_inference → spec_inference → proof_generation

rb_type_invariant_todo:   view_inference → view_refinement → inv_inference → spec_inference → proof_generation
                          (only one needing full sequence!)
```

---

## Action Items

### Immediate (Fix Current Issues)
1. ✅ **DONE:** Fix view_inference to handle `spec fn view` without deleting `spec` keyword
2. ✅ **DONE:** Implement surgical insertion (ask for implementation only, not full file)

### Short-term (Optimize Workflows)
3. ⏳ **TODO:** Update `prompts/plan_system.md` to add Simple View workflow
4. ⏳ **TODO:** Add detection for when view_refinement is actually needed
5. ⏳ **TODO:** Make proof_generation truly conditional (only when needed)

### Medium-term (Better Planning)
6. ⏳ **TODO:** Implement rule-based planner as Option 2 or 3
7. ⏳ **TODO:** Add benchmark-specific workflow overrides (config file?)
8. ⏳ **TODO:** Remove view_refinement from default workflows (make opt-in)

### Long-term (Validation)
9. ⏳ **TODO:** Run all 13 TODO benchmarks with optimized workflows
10. ⏳ **TODO:** Measure success rate improvement
11. ⏳ **TODO:** Measure time/cost savings from skipping unnecessary modules

---

## Expected Impact

### Time Savings
```
Current (Full Sequence):     5 modules × ~300s = 1500s average
Optimized (2-3 modules):     2.5 modules × ~300s = 750s average
Savings:                     50% time reduction
```

### Cost Savings
```
Current:  5 modules × LLM calls = high cost
Optimized: 2-3 modules × LLM calls = 40-50% cost reduction
```

### Success Rate
```
Current:  Many benchmarks fail due to unnecessary/wrong modules
Optimized: Higher success rate by running only needed modules
```

**Example:** `transfer_todo.rs` doesn't need view_inference or inv_inference. Running those modules wastes time and might introduce errors!
