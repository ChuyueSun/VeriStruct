# Examples-Based Teaching: Final Approach

**Philosophy:** Let examples do the teaching, not dynamic prompts
**Implementation:** 15 diverse examples with comprehensive inline guidance

---

## 🎯 **The Approach**

### **Don't:**
- ❌ Add dynamic guidance to prompts (clutters, confusing)
- ❌ Use benchmark-specific examples (overfitting)
- ❌ Rely on LLM to infer from generic terms

### **Do:**
- ✅ Create diverse educational examples
- ✅ Add comprehensive inline comments
- ✅ Show both correct and incorrect approaches
- ✅ Prioritize relevant examples via scoring

---

## 📚 **Examples Created (15 total)**

### **For Abstraction Level Teaching (4 new):**

1. **ex_abstract_simple.rs** - When abstract works
   - Simple container with Vec<T>
   - Shows abstract postconditions
   - Inline: "Use abstract when no encoding/packing"

2. **ex_concrete_packed.rs** - When concrete needed
   - Packed structure with Vec<u64>
   - Shows concrete postconditions with chunk extraction
   - Inline: "Use concrete when proof operates on chunks"

3. **ex_abstraction_comparison.rs** - Side-by-side comparison
   - Same operation, both levels
   - Shows when each works
   - Inline: Explains the difference

4. **ex_why_concrete.rs** - Educational deep-dive
   - Commented-out wrong approach
   - Working correct approach
   - Inline: Explains the verification chain step-by-step

### **Existing Examples (11 from before):**

5. **ex_bitmap.rs** - Generic abstraction patterns
6. **ex1.rs**, **ex2.rs** - Basic patterns
7. **ex_0_option_minimal.rs** - Option handling
8. **ex_atomic.rs** - Atomic operations
9. **ex_binary_search.rs** - Search algorithms
10. **ex_bst_option.rs** - Tree structures
11. **ex_isSome.rs** - Option predicates
12. **ex_seq.rs** - Sequence operations
13. **ex_type_bounds.rs** - Type constraints
14. **ex_vector_operations.rs** - Vector ops
15. **ex_vector_reverse.rs**, **ex_vector_swap.rs** - More vector patterns

---

## 🎯 **Smart Example Selection**

### **When Low-Level Patterns Detected:**

```python
if low_level_patterns['needs_concrete_specs']:
    # Educational examples get highest priority
    if 'why_concrete' in filename:
        score += 100  # Explains the WHY

    if 'abstraction_comparison' in filename:
        score += 100  # Shows both ways

    if 'concrete_packed' in filename:
        score += 90  # Shows the pattern

    if 'extract_component' in answer:
        score += 70  # Has the pattern
```

**Result:** Top 5 examples will be rich in abstraction level teaching!

---

## 📖 **What Each Example Teaches**

### **ex_abstract_simple.rs:**
```rust
// When to use ABSTRACT:
fn get(&self, index: usize) -> (elem: &T)
    ensures
        *elem == self@[index as int]  // ABSTRACT - works for simple structures
```

**Teaches:** Abstract is fine when no packing/encoding

### **ex_concrete_packed.rs:**
```rust
// When to use CONCRETE:
fn combine(&self, other: &PackedData) -> (result: PackedData)
    ensures
        forall|i: int| {
            let chunk_idx = i / COMPONENTS_PER_CHUNK;
            extract_component(result.chunks@[chunk_idx], ...) == ...
        }
```

**Teaches:** Concrete needed for packed structures with proofs

### **ex_why_concrete.rs:**
```rust
// Shows commented-out WRONG approach:
/*
fn combine_abstract(&self, other: &Self) -> (result: Self)
    ensures
        forall|i: int| result@[i] == ...  // UNPROVABLE!
*/

// Then shows CORRECT approach with explanation
fn combine_concrete(&self, other: &Self) -> (result: Self)
    ensures
        forall|i: int| {
            bit_is_set(result.chunks@[i/64], i%64) == ...
        }
```

**Teaches:** The verification chain and why concrete works

### **ex_abstraction_comparison.rs:**
```rust
// SCENARIO 1: Simple (abstract works)
impl SimpleContainer {
    fn merge(...) -> (result: ...)
        ensures forall|i: int| result@[i] == ...  // WORKS
}

// SCENARIO 2: Packed (concrete required)
impl PackedContainer {
    fn merge_wrong(...) -> (result: ...)
        // ensures forall|i: int| result@[i] == ...  // UNPROVABLE!

    fn merge_correct(...) -> (result: ...)
        ensures forall|i: int| {
            get_element_from_unit(result.units@[i/N], i%N) == ...  // WORKS!
        }
}
```

**Teaches:** Direct comparison, when to choose which

---

## 🎓 **Teaching Through Examples**

### **Inline Guidance in Every Example:**

All examples have extensive comments like:

```rust
// ========== WHEN TO USE CONCRETE POSTCONDITIONS ==========
//
// Use concrete (chunk-level) postconditions when:
// 1. Data is PACKED/ENCODED (multiple logical items per physical unit)
// 2. View EXPANDS underlying representation (chunks → components)
// 3. Proof functions operate on UNDERLYING type (chunks, not components)
//
// KEY PATTERN:
// - If view uses: extract_component(self.chunks@[i/N], i%N)
// - Then postcondition MUST use: extract_component(ret.chunks@[i/N], i%N)
// - NOT just: ret@[i]
//
// ==================================
```

**Benefits:**
- LLM sees guidance IN the examples
- No dynamic prompt modification needed
- Reusable across all cases
- Clean architecture

---

## 📊 **Expected Selection for bitmap_2_todo**

When `detect_low_level_patterns` finds bit-vector proofs:

**Top 5 examples (by score):**
1. `ex_why_concrete.rs` (+100) - Explains the verification chain
2. `ex_abstraction_comparison.rs` (+100) - Shows both approaches
3. `ex_concrete_packed.rs` (+90) - Shows concrete pattern
4. `ex_bitmap.rs` (+70) - Generic abstraction with extract_component
5. Other example with extract patterns (+60)

**All 5 will teach:** Use chunk-level postconditions for packed structures!

---

## ✅ **Advantages of This Approach**

### **1. No Overfitting**
- ✅ All examples use generic placeholders
- ✅ No benchmark-specific code
- ✅ Reusable across domains

### **2. Clean Architecture**
- ✅ Prompts stay simple
- ✅ No dynamic text injection
- ✅ Logic in scoring, not text generation

### **3. Rich Teaching**
- ✅ 4 examples teaching abstraction from different angles
- ✅ Inline comments explain WHY
- ✅ Shows both correct and incorrect

### **4. Scalable**
- ✅ Easy to add more examples
- ✅ Scoring adapts automatically
- ✅ No code changes needed for new patterns

---

## 🧪 **Testing Strategy**

### **Next Run Should:**

1. **Detect patterns** ✅
   - `has_bit_vector_proofs`: True
   - `needs_concrete_specs`: True

2. **Select examples:**
   - ex_why_concrete.rs (+100)
   - ex_abstraction_comparison.rs (+100)
   - ex_concrete_packed.rs (+90)
   - ex_bitmap.rs (+70)
   - (one more with extract patterns)

3. **LLM sees:**
   - Multiple examples showing extraction at chunk level
   - Inline comments explaining WHY
   - Both correct and incorrect approaches
   - Common pattern across all examples

4. **Expected result:**
   - LLM learns: "For packed structures, use extraction at chunk level"
   - Generates: `extract_component(ret.chunks@[i/N], i%N)` pattern
   - **Not:** `ret@[i]` pattern

---

## 📈 **Expected Impact**

### **If Examples-Based Teaching Works:**
- ✅ Clean, no overfitting
- ✅ Scalable to other patterns
- ✅ No code changes needed
- ✅ Validates example-driven learning

### **If It Doesn't Work:**
- Plan B: Surgical insertion (like view_inference)
- Ask for specs only, insert programmatically
- Most reliable approach

---

## ✨ **Summary**

**Created:** 3 new educational examples
**Updated:** Example scoring to prioritize them
**Removed:** Overfitted bitmap-specific example

**Total examples:** 15 (4 teaching abstraction levels)

**Approach:**
- ✅ Pattern detection → Example selection
- ✅ Examples teach through inline comments
- ✅ No dynamic prompt modification
- ✅ Generic, reusable patterns

**Philosophy:** Examples > Dynamic Guidance > Benchmark-Specific Code

**Status:** ✅ Ready for validation

---

## 🎯 **Files Summary**

### **New Examples:**
1. `ex_abstract_simple.rs` - When abstract works
2. `ex_concrete_packed.rs` - When concrete needed
3. `ex_abstraction_comparison.rs` - Side-by-side
4. `ex_why_concrete.rs` - Educational explanation

### **Updated:**
- `src/modules/spec_inference.py` - Enhanced example scoring

### **Removed:**
- `ex_bitmap_concrete.rs` - Was overfitting

**All examples are now generic and educational!** ✅
