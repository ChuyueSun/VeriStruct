# spec_inference Abstraction Guidance - Version 2 Improvements

**Problem:** Generic guidance wasn't specific enough for LLM to generate correct patterns
**Solution:** Make guidance domain-specific with exact code examples

---

## ❌ What Didn't Work (Version 1)

### **Generic Guidance:**

```
Use CONCRETE postconditions:
    extract_from_underlying(ret.underlying@[i/N], i%N) ==
    combine(extract_from_underlying(self.underlying@[i/N], i%N), ...)
```

### **Why it failed:**

- LLM saw `extract_from_underlying`
- Actual code uses `get_bit64!`
- **LLM couldn't translate generic to specific**
- Still generated: `ret@[i] == (self@[i] || ...)` ❌

---

## ✅ What Will Work (Version 2)

### **1. Specific Guidance with Actual Macros**

```python
if low_level_patterns['has_bit_vector_proofs']:
    abstraction_guidance += """
    **CRITICAL RULE: Postconditions MUST use get_bit64! macro (NOT abstract view @)**

    ✅ CORRECT - Concrete postcondition using get_bit64!:
    ```rust
    fn or(&self, other: &BitMap) -> (ret: BitMap)
        ensures
            forall|i: int| #![auto] 0 <= i < ret@.len() ==> {
                let chunk_i = i / 64;
                let bit_i = (i % 64) as u64;
                get_bit64!(ret.bits@[chunk_i], bit_i) ==
                (get_bit64!(self.bits@[chunk_i], bit_i) ||
                 get_bit64!(other.bits@[chunk_i], bit_i))
            }
    ```

    ❌ WRONG - Abstract postcondition (UNPROVABLE!):
    ```rust
    fn or(&self, other: &BitMap) -> (ret: BitMap)
        ensures
            forall|i: int| ret@[i] == (self@[i] || other@[i])  // TOO ABSTRACT!
    ```

    **PATTERN for ALL bitmap operations:**
    - Use: `get_bit64!(ret.bits@[i/64], (i%64) as u64)`
    - NOT: `ret@[i]`
    """
```

### **Why this works:**

- ✅ Shows EXACT macro name (`get_bit64!`)
- ✅ Shows EXACT pattern (`ret.bits@[i/64]`)
- ✅ Shows both correct and incorrect versions
- ✅ Explains WHY (connects to proof)
- ✅ Gives explicit rule to follow

---

## 📊 Comparison

| Aspect | Version 1 (Generic) | Version 2 (Specific) |
|--------|---------------------|----------------------|
| **Macro names** | `extract_from_underlying` | `get_bit64!` ✅ |
| **Field names** | `underlying` | `bits` ✅ |
| **Types** | `UnderlyingType` | `Vec<u64>` ✅ |
| **Concrete example** | Generic pattern | Actual bitmap code ✅ |
| **Explanation** | Abstract | Specific to bit-vectors ✅ |

---

## 🎯 Three-Pronged Approach

### **1. Specific Guidance** ✅ (Just implemented)

- Detects bit-vector patterns
- Shows EXACT `get_bit64!` pattern
- Not generic abstractions

### **2. Specific Examples** ✅ (Already created)

- `ex_bitmap_concrete.rs` with get_bit64! macros
- Scored +100 when `get_bit64!` detected
- Will bubble to top of examples

### **3. Enhanced Scoring** ✅ (Already implemented)

```python
if 'get_bit64!' in answer and ('storage' in answer or 'bits' in answer):
    score += 100  # Exact pattern match!
```

---

## 🚀 Expected Impact

### **Before (Version 1):**

- Detection: ✅ Working
- Guidance: ⚠️ Generic (`extract_from_underlying`)
- Examples: ⚠️ Generic (`ex_bitmap.rs`)
- Result: ❌ LLM generates abstract

### **After (Version 2):**

- Detection: ✅ Working
- Guidance: ✅ Specific (`get_bit64!` with exact code)
- Examples: ✅ Specific (`ex_bitmap_concrete.rs` +100 score)
- Result: ✅ **LLM should generate concrete!**

---

## 📋 Complete Pattern Coverage

### **For Bit-Vector Operations:**

**Detected patterns:**

- `#[verifier::bit_vector]`
- `bit_or_64_proof`, `set_bit64_proof`
- `get_bit64!`, `set_bit64!`
- `Vec<u64>` + `Seq<bool>`

**Guidance added:**

- ✅ Explicit: "MUST use get_bit64! macro"
- ✅ Concrete example with actual macros
- ✅ Shows both right and wrong
- ✅ Explains why (proof connection)
- ✅ Gives pattern to follow

**Examples prioritized:**

- ✅ `ex_bitmap_concrete.rs` (+100 score)
- ✅ Any example with `get_bit64!` (+100)
- ⏭️ Generic examples (+60 as fallback)

---

## 🧪 Testing

### **Validation Steps:**

1. **Run bitmap_2_todo:**

   ```bash
   VERUS_TEST_FILE=benchmarks-complete/bitmap_2_todo.rs python3 -m src.main
   ```

2. **Check logs for:**
   - "Detected low-level patterns: ...bit_vector_proofs..." ✅
   - "Bitmap-specific example found (+100)"
   - "Prioritized abstraction-level examples"

3. **Check prompts:**
   - Verify guidance includes `get_bit64!` (not `extract_*`)
   - Verify ex_bitmap_concrete.rs in examples

4. **Check generated code:**
   - `fn or` postcondition uses `get_bit64!` ✅
   - `fn set_bit` postcondition uses `get_bit64!` ✅
   - `fn get_bit` postcondition uses `get_bit64!` ✅

5. **Expected result:**
   - Verified: 5-6 (after spec_inference)
   - Then 7 after proof_generation
   - 100% verification! ✅

---

## 💡 Key Improvements in Version 2

### **1. Domain Detection → Domain-Specific Guidance**

**Old:**

```python
if needs_concrete:
    add_generic_guidance()  # Same for all domains
```

**New:**

```python
if has_bit_vector_proofs:
    add_bitmap_specific_guidance()  # get_bit64! macros
elif has_other_pattern:
    add_other_specific_guidance()  # Pattern-specific
else:
    add_generic_guidance()  # Fallback
```

### **2. Show Actual Code, Not Abstractions**

**Old:** `extract_from_underlying(...)` (LLM must translate)
**New:** `get_bit64!(ret.bits@[i/64], ...)` (LLM can copy directly)

### **3. Concrete Examples in Guidance**

**Old:** "Study the examples"
**New:** Full correct + incorrect examples IN the guidance itself

### **4. Explicit Rules**

**Old:** General principle
**New:** "Use `get_bit64!(...)`" "NOT `ret@[i]`"

---

## 🎓 Lessons for LLM Guidance

### **What Works:**

1. ✅ **Show, don't tell** - Concrete code examples > Abstract descriptions
2. ✅ **Be specific** - Use actual macro/function names from the code
3. ✅ **Show both ways** - Correct AND incorrect examples
4. ✅ **Explain why** - Connect to proof functions
5. ✅ **Give rules** - Explicit "DO" and "DON'T"

### **What Doesn't Work:**

1. ❌ **Generic abstractions** - `extract_*` when code uses specific macros
2. ❌ **Indirect guidance** - "Match proof level" without showing how
3. ❌ **Rely on inference** - LLM won't make connections automatically
4. ❌ **Examples alone** - Need guidance + examples together

---

## 🔄 If This Still Doesn't Work

### **Backup Plan: Surgical Insertion (Like view_inference)**

Apply the proven surgical insertion approach to spec_inference:

```python
# 1. Detect function signatures
functions = extract_function_signatures(code)

# 2. Ask LLM for just requires/ensures for each function
for func in functions_with_todo:
    spec = llm.generate_specs_for_function(
        func,
        guidance="Use get_bit64! for bitmap operations"
    )

# 3. Insert surgically
final_code = insert_specs(original_code, specs)
```

**Advantages:**

- LLM can't modify other parts
- Can provide function-specific templates
- More reliable than whole-file approach
- Proven to work for view_inference

---

## ✨ Summary

**Version 1:**

- Generic guidance + generic examples
- LLM couldn't translate to specific patterns
- Failed to generate concrete postconditions

**Version 2:**

- Specific guidance (actual `get_bit64!` macros)
- Specific examples (`ex_bitmap_concrete.rs`)
- Enhanced scoring (+100 for exact matches)
- **Should work!** ⏳

**If Version 2 fails:**

- Apply surgical insertion (proven approach)
- Most reliable solution

---

**Status:**

- ✅ Guidance improved (now bitmap-specific)
- ✅ Examples created (ex_bitmap_concrete.rs)
- ✅ Scoring enhanced (+100 for get_bit64!)
- ⏳ Ready for testing

**Next:** Test on fresh run and validate!
