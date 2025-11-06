# Abstraction Level Fix - Diagnosis (Run: azure_20251105_145846)

**Status:** ❌ **NOT WORKING YET**

---

## What Happened

### ✅ Detection Worked
From log line 566-567:
```
Detected low-level patterns: ['has_bit_vector_proofs', 'has_packed_structure', 'has_low_level_ops', 'needs_concrete_specs']
Will prioritize examples with concrete postconditions
```

### ✅ Guidance Added
The prompts show:
```
**DETECTED: LOW-LEVEL/PACKED STRUCTURE PATTERNS**

This code uses low-level operations with proof functions.

**CRITICAL: Postconditions must match proof function level!**
```

### ❌ But LLM Still Generated Abstract Postconditions

**What it generated:**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    ensures
        bit == self@[index as int]  // ABSTRACT - unprovable!
```

**What it should have generated:**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    ensures
        bit == get_bit64!(self.bits@[(index/64) as int], (index%64) as u64)  // CONCRETE - provable!
```

---

## Root Cause

**The problem:** Generic examples don't translate to specific bitmap patterns

### What We Have:
- Generic guidance: "Use `extract_from_underlying(ret.underlying@[i/N], i%N)`"
- Generic example in `ex_bitmap.rs`: Uses `extract_component`, `UnderlyingType`

### What LLM Sees:
- "Use concrete postconditions... with extract_from_underlying..."
- But the actual code uses `get_bit64!`, not `extract_from_underlying`
- LLM doesn't make the connection!

### Gap:
**LLM doesn't know that:**
```
extract_from_underlying(...)  →  translates to  →  get_bit64!(...)
```

---

## Solution

### Created: Specific Bitmap Example ✅

**File:** `src/examples/output-requires/ex_bitmap_concrete.rs`

**Shows exactly:**
```rust
fn read_bit(&self, idx: u32) -> (result: bool)
    requires
        (idx as nat) < self@.len()
    ensures
        // CONCRETE: Use get_bit64! to match the view definition
        result == get_bit64!(self.storage@[(idx / 64) as int], (idx % 64) as u64)
```

**And:**
```rust
fn combine(&self, other: &S) -> (result: S)
    ensures
        forall|i: int| #![auto] 0 <= i < result@.len() ==> {
            let unit_i = i / 64;
            let bit_i = (i % 64) as u64;
            get_bit64!(result.storage@[unit_i], bit_i) ==
            (get_bit64!(self.storage@[unit_i], bit_i) ||
             get_bit64!(other.storage@[unit_i], bit_i))
        }
```

This is the **EXACT pattern** bitmap_2_todo needs!

---

## Why This Will Work

### Before (too generic):
- Examples use: `extract_from_underlying`, `extract_component`
- LLM sees generic pattern
- Doesn't know how to apply to `get_bit64!`
- Generates abstract `ret@[i]` instead

### After (specific):
- Example uses: `get_bit64!` directly
- LLM sees exact pattern needed
- Can copy/adapt the pattern
- Will generate concrete postconditions! ✅

---

## Implementation Status

### ✅ Completed:
1. Pattern detection in spec_inference
2. Dynamic guidance injection
3. Generic abstraction examples (`ex_bitmap.rs`)
4. Specific bitmap example (`ex_bitmap_concrete.rs`)

### ⏳ Still Needed:
1. **Make sure ex_bitmap_concrete.rs is included in examples**
   - It's in `output-requires/` directory
   - Should be picked up by `get_examples(config, "requires", ...)`
   - But needs to be prioritized for bitmap code

2. **Increase scoring for specific examples**
   - When code has `get_bit64!`, boost `ex_bitmap_concrete.rs` score massively
   - Current: Generic examples get +60
   - Should be: Specific bitmap example gets +100

---

## Fix Required

Update example selection in `spec_inference.py`:

```python
# In example selection loop
if low_level_patterns['needs_concrete_specs']:
    # Existing: Generic pattern matching
    if 'extract_' in answer or '_from_unit' in answer:
        score += 60

    # ADD: Specific bitmap pattern matching (highest priority!)
    if low_level_patterns['has_bit_vector_proofs']:
        if 'get_bit64!' in answer and 'Vec<u64>' in answer:
            score += 100  # Highest priority for exact pattern match!
```

This will ensure `ex_bitmap_concrete.rs` bubbles to the top when bitmap patterns detected!

---

## Expected Result After Fix

### Before (Current):
- Detection: ✅ Working
- Guidance: ✅ Added
- Examples: ❌ Too generic
- Result: ❌ Abstract postconditions

### After (With Specific Example):
- Detection: ✅ Working
- Guidance: ✅ Added
- Examples: ✅ Specific (ex_bitmap_concrete.rs)
- Result: ✅ Concrete postconditions

---

## Testing Plan

1. Update example scoring to prioritize `ex_bitmap_concrete.rs`
2. Run bitmap_2_todo again
3. Check prompts to verify ex_bitmap_concrete.rs is included
4. Verify generated postconditions use `get_bit64!`
5. Expected: V=7/7 (100%) instead of V=4/7

---

## Lesson Learned

**Generic examples + generic guidance ≠ Specific application**

The LLM needs to see the **EXACT pattern** it should use:
- ✅ Specific macro names (`get_bit64!` not `extract_*`)
- ✅ Specific types (`Vec<u64>` not `UnderlyingType`)
- ✅ Specific operations (bit-vector proofs)

**For domain-specific patterns, domain-specific examples are essential!**

---

## Action Items

**Immediate:**
1. ⏳ Update scoring in spec_inference.py to prioritize ex_bitmap_concrete.rs
2. ⏳ Test on bitmap_2_todo
3. ⏳ Verify it works

**If It Works:**
- Create similar specific examples for other domains
- Build library of domain-specific patterns
- Keep generic examples as fallback

**If It Still Doesn't Work:**
- May need even more explicit guidance
- Or surgical insertion for spec_inference too (like view_inference)
- Or hardcode bitmap patterns as special case
