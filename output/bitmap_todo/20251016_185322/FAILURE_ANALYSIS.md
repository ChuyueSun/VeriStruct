# Bitmap Run 20251016_185322 - Failure Analysis

**Status**: ❌ FAILED - Compilation Error (Syntax)
**Final Score**: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1

---

## Root Cause: Incorrect Syntax in spec_inference

The benchmark failed during the **repair stage** due to a **syntax error** introduced by the spec_inference module.

### The Error

```
error: expected `,`
   --> repair_round_1_bitmap_todo__Map_General_20251016_185322.rs:131:32
    |
131 |             index as int < self.view().len(),
    |                                ^
```

### What Went Wrong

The **spec_inference module** used `.view()` explicitly in specifications instead of the `@` shorthand notation required by Verus:

**❌ Generated (WRONG - causes syntax error):**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index as int < self.view().len(),  // ERROR: Can't call .view() in specs
    ensures
        bit == self.view()[index as int],
```

**✅ Ground Truth (CORRECT):**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index < self@.len(),  // Use @ shorthand
    ensures
        bit == self@[index as int],
```

### Why This Happened

The spec_inference prompt had **incorrect instructions** telling the LLM to use `.view()`:

**Lines 52-53 in spec_inference.py (BEFORE FIX):**
```python
"     * If type T implements View: use `self.view().field` to access fields\n"
"     * For tuples returned by view(): if `self.view()` returns (A, B), use `self.view().0`, `self.view().1`\n"
```

### Scope of Impact

The `.view()` syntax error appeared in **13 locations** across the generated code:
- `get_bit` function requires/ensures
- `set_bit` function requires/ensures
- `or` function requires/ensures
- `from` function ensures
- Inside proof block comments

---

## Good News: View Specification is NOW CORRECT! ✅

The earlier fix to the proof_generation prompt **worked successfully**. The view specification now uses `Seq<bool>`:

**Generated (CORRECT):**
```rust
spec fn view(&self) -> Seq<bool> {
    Seq::new((self.bits@.len() * 64) as nat, |idx: int| {
        ((self.bits@[(idx / 64)] >> (idx % 64)) & 0x1) == 1
    })
}
```

This matches the ground truth pattern and is a huge improvement over the previous `(nat, Set<nat>)` approach!

---

## The Fix Applied

Updated **src/modules/spec_inference.py** to correct the instructions:

### Change 1: Main requires/ensures instructions (lines 51-57)

**BEFORE:**
```python
"   - For field access in specifications of public functions:\n"
"     * If type T implements View: use `self.view().field` to access fields\n"
"     * For tuples returned by view(): if `self.view()` returns (A, B), use `self.view().0`, `self.view().1`\n"
```

**AFTER:**
```python
"   - **CRITICAL: For types with spec fn view(), use @ shorthand in specifications:**\n"
"     * ALWAYS use `self@` instead of `self.view()` in requires/ensures\n"
"     * ALWAYS use `ret@` instead of `ret.view()` in ensures\n"
"     * ALWAYS use `old(self)@` instead of `old(self).view()` in ensures\n"
"     * Examples: `self@.len()`, `self@.field`, `ret@[i]`, `old(self)@[i]`\n"
"     * For tuples: if view() returns (A, B), use `self@.0`, `self@.1`\n"
"     * NEVER write `self.view()` directly - it causes syntax errors\n"
```

### Change 2: Trait implementation instructions (lines 68-70)

**BEFORE:**
```python
"     * If type implements View: use `self.view().field`\n"
```

**AFTER:**
```python
"     * If type implements View: use `self@.field` (NOT `self.view().field`)\n"
```

### Change 3: Field Access Rules section (lines 87-91)

**BEFORE:**
```python
"   - For types with View: use `self.view().field`\n"
"   - For tuple views: use `self.view().0`, `self.view().1`, etc.\n"
"     * CORRECT: `(x as nat) < (self.view().0)`\n"
```

**AFTER:**
```python
"   - For types with View: use `self@.field` (the @ is shorthand for .view())\n"
"   - For tuple views: use `self@.0`, `self@.1`, etc.\n"
"     * CORRECT: `(x as nat) < (self@.0)`\n"
```

---

## Expected Behavior After Fix

With both fixes in place:

1. ✅ **view() will use Seq<bool>** (from proof_generation prompt update)
2. ✅ **Specifications will use @ notation** (from spec_inference prompt update)
3. ✅ **Proof blocks will use assert_seqs_equal!** (from proof_generation prompt update)

The next run should generate code like:

```rust
spec fn view(&self) -> Seq<bool> { ... }

fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index < self@.len(),  // ✅ Using @
    ensures
        bit == self@[index as int],
{
    ...
}

fn set_bit(&mut self, index: u32, bit: bool)
    requires
        index < old(self)@.len(),  // ✅ Using old(self)@
    ensures
        self@ == old(self)@.update(index as int, bit),
{
    ...
    proof {
        assert_seqs_equal!(  // ✅ Using assert_seqs_equal!
            self.view(),
            old(self).view().update(index as int, bit)
        );
    }
}
```

---

## Summary of All Fixes Made

### Fix #1: Proof Generation - Add assert_seqs_equal! knowledge
- **File**: `src/prompts/verus_proof.md`
- **Lines**: 22-35
- **Impact**: LLM now knows to use `assert_seqs_equal!` macro for sequence updates
- **Status**: ✅ Applied and documented

### Fix #2: Spec Inference - Use @ notation instead of .view()
- **File**: `src/modules/spec_inference.py`
- **Lines**: 51-57, 68-70, 87-94
- **Impact**: LLM now generates `self@` instead of `self.view()` in specifications
- **Status**: ✅ Applied in this session

---

## Next Steps

1. **Rerun bitmap benchmark** with fresh cache to test both fixes
2. **Verify the generated code** compiles without syntax errors
3. **Check verification results** - expect all functions to verify with proper Seq<bool> view and assert_seqs_equal!
4. **Compare against ground truth** to ensure pattern matching

The combination of these two fixes should resolve the bitmap verification failure completely.
