# Benchmark Patterns Analysis

## Question: Do all benchmarks fit the current module processing pattern?

**Answer: NO** - Benchmarks have different patterns requiring different module workflows.

---

## Current Full Module Workflow
```
view_inference → view_refinement → inv_inference → spec_inference → proof_generation
```

**Problem:** Not all benchmarks need view functions!

---

## Benchmark Categories

### **Category 1: NO VIEW NEEDED** ❌ View modules not applicable

#### 1a. Simple Functions Only
- **Files:** `transfer_todo.rs`, `vectors_todo.rs`
- **Pattern:** Standalone functions with no structs
- **Needs:**
  - ✅ spec_inference (requires/ensures)
  - ✅ proof_generation (loop invariants, proofs)
- **Skip:** view_inference, view_refinement, inv_inference

**Example (transfer_todo.rs):**
```rust
pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
// TODO: add requires and ensures
```

#### 1b. Trait Implementations Only
- **Files:** `invariants_todo.rs`, `rwlock_vstd_todo.rs`
- **Pattern:** Trait impl with spec functions needing bodies
- **Needs:**
  - ✅ spec_inference (fill in trait spec functions)
- **Skip:** view_inference, view_refinement, inv_inference, proof_generation

**Example (invariants_todo.rs):**
```rust
impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        // TODO: add specification
    }
}
```

#### 1c. Enums with Spec Functions
- **Files:** `option_todo.rs`
- **Pattern:** Enum with helper spec functions
- **Needs:**
  - ✅ spec_inference (requires/ensures, spec function bodies)
- **Skip:** view_inference, view_refinement, inv_inference

**Example (option_todo.rs):**
```rust
pub enum MyOption<A> { None, Some(A) }

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    // TODO: add specification
}
```

#### 1d. Struct with Type Invariants (No View)
- **Files:** `atomics_todo.rs`, `node_todo.rs`
- **Pattern:** Struct with `#[verifier::type_invariant]` or spec functions, but no view
- **Needs:**
  - ✅ inv_inference (type invariants)
  - ✅ spec_inference (requires/ensures, spec function bodies)
  - ✅ proof_generation (proofs in loops/atomics)
- **Skip:** view_inference, view_refinement

**Example (atomics_todo.rs):**
```rust
struct Lock<T> {
    spec fn well_formed(&self) -> bool {
        // TODO: add specification
    }
}
```

---

### **Category 2: VIEW - spec fn style** ✅ Fill in existing spec fn body

#### 2a. Simple spec fn view
- **Files:** `bitmap_2_todo.rs`, `bitmap_todo.rs`, `set_from_vec_todo.rs`
- **Pattern:** Has `spec fn view(&self) -> Type` or `closed spec fn view` inside impl block with TODO
- **Needs:**
  - ✅ view_inference (**spec fn body filling mode**)
  - ✅ spec_inference (requires/ensures for other methods)
  - ✅ proof_generation (proofs)
- **Skip:** view_refinement (not needed for simple spec fn)
- **Maybe:** inv_inference (if struct has type invariants)

**Example (bitmap_2_todo.rs):**
```rust
impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        // TODO: Implement the view function
    }
}
```

**Critical:** View inference must detect this pattern and **ONLY fill in the body**, not convert to `impl View for`!

---

### **Category 3: VIEW - View trait style** ✅ Implement View trait

#### 3a. Empty impl View for
- **Files:** `rb_type_invariant_todo.rs`
- **Pattern:** Has `impl View for StructName { // TODO }` with completely empty impl
- **Needs:**
  - ✅ view_inference (**View trait implementation mode**)
  - ✅ view_refinement (may need refinement)
  - ✅ inv_inference (RingBuffer has type invariants)
  - ✅ spec_inference (requires/ensures)
  - ✅ proof_generation (proofs)

**Example (rb_type_invariant_todo.rs):**
```rust
impl<T: Copy> View for RingBuffer<T> {
    // TODO: add specification
}
```

#### 3b. impl View for with TODO in view function
- **Files:** `bst_map_todo.rs`, `treemap_todo.rs`
- **Pattern:** Has `impl View for` with `type V` but view function has TODO
- **Needs:**
  - ✅ view_inference (**fill in view function within existing View trait**)
  - ✅ inv_inference (TreeMap has type invariants)
  - ✅ spec_inference (requires/ensures)
  - ✅ proof_generation (proofs)

**Example (bst_map_todo.rs):**
```rust
impl<V> View for TreeMap<V> {
    type V = Map<u64, V>;

    open spec fn view(&self) -> Map<u64, V> {
        // TODO: add specification
    }
}
```

---

## Summary Statistics

| Category | Count | Example Files |
|----------|-------|---------------|
| No View (functions only) | 2 | transfer, vectors |
| No View (traits only) | 2 | invariants, rwlock |
| No View (enums) | 1 | option |
| No View (struct with inv) | 2 | atomics, node |
| View - spec fn style | 3 | bitmap_2, bitmap, set_from_vec |
| View - View trait (empty) | 1 | rb_type_invariant |
| View - View trait (partial) | 2 | bst_map, treemap |

**Total:** 13 TODO benchmarks with **7 different workflow patterns**!

---

## Required Changes

### 1. **Planning Module Must Detect Pattern**

The planning/workflow selection needs to:
- ✅ Detect if code has a struct/enum/trait
- ✅ Detect if code has View (spec fn vs trait style)
- ✅ Detect if code has type invariants
- ✅ Select appropriate module sequence

### 2. **View Inference Module Must Handle 3 Cases**

Current implementation already handles:
- ✅ **Case A:** spec fn view with TODO → fill in body
- ✅ **Case B:** impl View for (empty) → implement complete trait
- ❓ **Case C:** impl View for with TODO in view function → fill in just the view function

Need to add Case C detection!

### 3. **Conditional Module Execution**

Modules should be executed conditionally:
```python
workflow = []

if needs_view_inference():
    workflow.append("view_inference")
    if is_complex_view():  # Complex structs may benefit from refinement
        workflow.append("view_refinement")

if has_type_invariants():
    workflow.append("inv_inference")

workflow.append("spec_inference")  # Always needed for requires/ensures

if has_proofs_or_loops():
    workflow.append("proof_generation")

return workflow
```

### 4. **Benchmark-Specific Workflow Examples**

```
transfer_todo.rs:       spec_inference → proof_generation
invariants_todo.rs:     spec_inference
option_todo.rs:         spec_inference
atomics_todo.rs:        inv_inference → spec_inference → proof_generation
bitmap_2_todo.rs:       view_inference → spec_inference → proof_generation
rb_type_invariant:      view_inference → view_refinement → inv_inference → spec_inference → proof_generation
bst_map_todo.rs:        view_inference → inv_inference → spec_inference → proof_generation
```

---

## Critical Finding: Abstraction Level Matters

### The Postcondition Problem

Analysis of bitmap_2_todo reveals a **critical spec_inference issue**:

**Generated (unprovable):**
```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    ensures
        forall|i: int| ret@[i] == (self@[i] || bm@[i])  // ABSTRACT level
```

**Correct (provable):**
```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    ensures
        forall|i: int| 0 <= i < ret@.len() ==>
            get_bit64!(ret.bits@[i/64], (i%64) as u64) ==  // CONCRETE level
            (get_bit64!(self.bits@[i/64], (i%64) as u64) || ...)
```

### Why This Matters

When operations use **concrete-level proof functions** (like `bit_or_64_proof`):
- ❌ Abstract postconditions create an **abstraction gap** (unprovable)
- ✅ Concrete postconditions **match the proof** (provable)

### Affected Benchmarks

**Need concrete postconditions:**
- `bitmap_2_todo.rs` - Uses bit_or_64_proof, set_bit64_proof
- `bitmap_todo.rs` - Uses bit_or_64_proof, set_bit64_proof
- Any benchmark with bit-vector operations

**Can use abstract postconditions:**
- `bst_map_todo.rs` - Map operations, no bit-level proofs ✅
- `set_from_vec_todo.rs` - Set operations ✅
- Most other benchmarks ✅

### Impact

**Current bitmap results:**
- bitmap_2_todo: V=6/7 (85%) - postcondition unprovable
- bitmap_todo: V=5/7 (71%) - similar issue

**With concrete postconditions:**
- bitmap_2_todo: V=7/7 (100%) ✅
- bitmap_todo: V=7/7 (100%) ✅

**Success rate improvement: +15-29% for bitmap benchmarks!**

### Solution

1. Update `spec_inference` instruction to teach abstraction level selection
2. Add examples showing concrete vs abstract patterns
3. Add pattern detection for when to use concrete postconditions

See: `abstraction_level_guide.md` for detailed analysis and solutions.

---

## Conclusion

**The current "Full Sequence Workflow" is TOO HEAVY for most benchmarks!**

Only `rb_type_invariant_todo.rs` actually needs the full 5-module sequence. Most benchmarks need 1-3 modules.

**Additional Finding:** spec_inference needs to understand abstraction levels for proof-heavy code.

**Recommendations:**
1. Implement intelligent workflow planning that selects only the necessary modules
2. Fix spec_inference to generate concrete postconditions for bit-vector operations
3. Add examples demonstrating abstraction level selection
