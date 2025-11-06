# Final Approach: Teaching Through Examples (Not Dynamic Guidance)

**Principle:** Let examples teach the patterns, not prompts

---

## ✅ **What We Did**

### **Removed: Dynamic Guidance in Code**

**Before:**
```python
if low_level_patterns['needs_concrete_specs']:
    # Add 30 lines of guidance to prompt dynamically
    abstraction_guidance = "..."
    instruction += abstraction_guidance
```

**After:**
```python
# Just detect patterns and select examples - NO dynamic guidance!
patterns = detect_low_level_patterns(code)

# Let example selection do the work
if patterns['has_bit_vector_proofs'] and 'get_bit64!' in example:
    score += 100  # Prioritize relevant examples
```

**Why this is better:**
- ✅ Keeps prompts clean and focused
- ✅ Examples are self-contained teaching materials
- ✅ LLM learns from patterns, not instructions
- ✅ Less token usage
- ✅ More maintainable (examples in one place)

---

## 📚 **How It Works: Example-Driven Learning**

### **1. Pattern Detection (in code)**
```python
patterns = detect_low_level_patterns(code)
# Detects: bit_vector_proofs, packed_structures, low_level_ops
```

### **2. Example Scoring (in code)**
```python
if patterns['has_bit_vector_proofs']:
    if 'get_bit64!' in example and 'storage' in example:
        score += 100  # Exact match!
    elif 'concrete' in example_file:
        score += 70
```

### **3. Example Selection (automatic)**
```
Top 5 examples by score:
  1. ex_bitmap_concrete.rs (+100) ← Specific bit-vector pattern
  2. ex_bitmap.rs (+70) ← Generic abstraction guidance
  3. ... (other high-scoring examples)
```

### **4. LLM Learns (from examples)**
LLM sees `ex_bitmap_concrete.rs`:
```rust
// Shows: get_bit64!(ret.storage@[i/64], (i%64) as u64)
// Comment explains: "Use extraction macro at chunk level"
// Comment shows wrong way: ret@[i] ← Creates abstraction gap!
```

LLM copies the pattern! ✅

---

## 📁 **Examples Teach Everything**

### **ex_bitmap.rs (Generic)**

**Shows:**
- Abstract postconditions for simple operations
- Concrete postconditions for packed structures
- When to use each

**Inline comments explain:**
```rust
// ========== PATTERN 1: ABSTRACT LEVEL (Standard Operations) ==========
fn size(&self) -> (result: usize)
    ensures
        result == self@.len(),  // ABSTRACT - expresses intent clearly

// ========== PATTERN 2: CONCRETE LEVEL (Low-Level Proofs) ==========
fn modify_component(&mut self, idx: usize, new_value: LogicalValue)
    ensures
        // CONCRETE - matches what low_level_proof establishes!
        forall|i: int| #![auto] extract_component(self.underlying@[i/N], i%N) == ...
```

**Bottom section:**
```rust
// **The Verification Chain:**
// 1. Operation: low_level_operation(...)
// 2. Proof call: low_level_proof(...)
// 3. Proof establishes: extract_component(...)
// 4. Postcondition MUST match: extract_component(...)
// 5. Result: Verus can connect proof to postcondition ✓
```

### **ex_bitmap_concrete.rs (Specific)**

**Shows:**
- Actual bit-vector operations with macros
- Concrete pattern with get_bit64!
- Exactly what bitmap code needs

**Inline comments:**
```rust
// ========== CONCRETE POSTCONDITION FOR or ==========
fn combine(&self, other: &S) -> (result: S)
    ensures
        // CONCRETE: Use get_bit64! to match what bit_or_64_proof establishes
        forall|i: int| #![auto] 0 <= i < result@.len() ==> {
            get_bit64!(result.storage@[unit_i], bit_i) == ...
        }
```

**Bottom section:**
```rust
// ========== KEY PATTERN ==========
// For structures with Vec<u64> storage and Seq<bool> view:
// ALWAYS use get_bit64! in postconditions
// DO NOT use abstract view: ret@[i] ← Creates abstraction gap!
```

---

## 🎯 **The Complete Flow**

```
Code arrives with get_bit64! and bit_or_64_proof
                ↓
detect_low_level_patterns()
                ↓
{has_bit_vector_proofs: True}
                ↓
Example scoring:
  ex_bitmap_concrete.rs: +100 (has get_bit64!)
  ex_bitmap.rs: +70 (has concrete pattern)
  others: +0 to +50
                ↓
Top 5 examples selected (bitmap ones at top)
                ↓
LLM sees:
  - ex_bitmap_concrete.rs with get_bit64! pattern
  - ex_bitmap.rs explaining abstraction levels
  - Clear inline comments in examples
                ↓
LLM learns:
  "Use get_bit64!(ret.storage@[i/64], ...) not ret@[i]"
                ↓
Generates correct concrete postcondition! ✅
```

---

## ✅ **Advantages of Example-Only Approach**

### **vs. Dynamic Guidance:**

| Aspect | Dynamic Guidance | Example-Only | Winner |
|--------|------------------|--------------|--------|
| **Prompt size** | +30 lines per detection | No change | ✅ Examples |
| **Maintainability** | Scattered in code | Centralized in examples | ✅ Examples |
| **Clarity** | Text explanation | Code demonstration | ✅ Examples |
| **Token usage** | Higher | Lower | ✅ Examples |
| **LLM learning** | From instructions | From patterns | ✅ Examples |
| **Extensibility** | Add more code | Add more examples | ✅ Examples |

### **Why Examples Work Better:**

1. ✅ **Show, don't tell** - Code is clearer than prose
2. ✅ **Self-contained** - Each example is complete
3. ✅ **Pattern-based** - LLMs excel at pattern matching
4. ✅ **Maintainable** - Easy to add/modify examples
5. ✅ **Scalable** - Just add more examples for new patterns

---

## 📊 **Implementation Status**

### **Completed:**

1. ✅ **Removed dynamic guidance** from spec_inference.py
2. ✅ **Created generic example** (ex_bitmap.rs) with clear guidance comments
3. ✅ **Created specific example** (ex_bitmap_concrete.rs) with get_bit64! patterns
4. ✅ **Enhanced example scoring** (+100 for exact pattern matches)
5. ✅ **Pattern detection** (identifies when examples needed)

### **How It Works Now:**

```python
# In spec_inference.py - CLEAN AND SIMPLE:

# 1. Detect patterns
patterns = detect_low_level_patterns(code)

# 2. Score examples (prioritize relevant ones)
for example in all_examples:
    if patterns['has_bit_vector_proofs']:
        if 'get_bit64!' in example:
            score += 100  # Exact match!

# 3. Select top 5 examples
top_examples = sort_by_score(examples)[:5]

# 4. Let LLM learn from examples (no extra guidance needed!)
```

**That's it!** No dynamic prompt modification, just smart example selection.

---

## 🎓 **Lesson Learned**

**Don't add guidance to prompts - add it to examples!**

**Bad approach:**
- Detect pattern → Add guidance to prompt → Hope LLM follows

**Good approach:**
- Detect pattern → Select relevant examples → LLM learns naturally

**Why:**
- Examples are clearer than instructions
- LLMs are better at pattern matching than following rules
- Examples are reusable and maintainable
- Less coupling between code and prompts

---

## ✨ **Summary**

**Changed from:**
- Dynamic guidance injection (30+ lines added to prompt)
- Generic examples only
- LLM must translate guidance to code

**Changed to:**
- No dynamic guidance
- Smart example selection (scoring +100 for exact matches)
- Examples teach through clear inline comments
- LLM copies patterns directly

**Result:**
- ✅ Cleaner code (no guidance strings in spec_inference.py)
- ✅ Better teaching (examples show, not tell)
- ✅ More maintainable (examples in one place)
- ✅ Ready for testing

---

## 🚀 **Ready to Test**

**Current state:**
- ✅ Pattern detection: Working
- ✅ Example selection: Working (+100 for get_bit64!)
- ✅ Examples: Self-documenting with clear comments
- ⏳ LLM learning: Ready to validate

**Next run should:**
- Select ex_bitmap_concrete.rs (highest score)
- LLM sees get_bit64! pattern
- Generates concrete postconditions
- **Expected: Verified 7/7!** ✅

**No more dynamic guidance - let examples do the teaching!** 🎯
