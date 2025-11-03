# Side-by-Side Comparison: Broken vs Fixed

## Overview

| Metric | Broken (final_result.rs) | Fixed (fixed_bitmap.rs) |
|--------|--------------------------|-------------------------|
| **Compilation** | ❌ Failed | ✅ Success |
| **Verified** | -1 | 8 |
| **Errors** | 999 | 0 |
| **Verus Errors** | 1 (syntax) | 0 |

---

## The Critical Difference: `or()` Method Proof

### ❌ BROKEN VERSION

```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self@.len() == bm@.len()
    ensures
        ret@.len() == self@.len(),
        forall|i: int| 0 <= i && i < self@.len() ==> ret@[i] == (self@[i] || bm@[i])
{
    let n: usize = self.bits.len();
    let mut i: usize = 0;
    let mut res_bits: Vec<u64> = Vec::new();
    let mut result = BitMap { bits: res_bits };
    while i < n
        invariant
            0 <= i as int <= n as int,
            result.bits@.len() == i as int,
            self@.len() == bm@.len(),
            forall|k: int| 0 <= k && k < (i as int)*64 ==>
                result@[k] == self@[k] || bm@[k],
        decreases n - i
    {
        res_bits = result.bits;
        let u1: u64 = self.bits[i];
        let u2: u64 = bm.bits[i];
        let or_int: u64 = u1 | u2;
        res_bits.push(or_int);
        result = BitMap { bits: res_bits };

        proof {
            bit_or_64_proof(u1, u2, or_int);
            assert forall|off: int| #![trigger result@[(i as int) * 64 + off]]
                0 <= off && off < 64 ==>
                    result@[(i as int) * 64 + off]
                        == (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off]);
            //                                                                         ^
            //                                                                         |
            //                                                      SYNTAX ERROR HERE: missing `by`
        }

        i = i + 1;
    }
    result
}
```

**Error**: Line 144, expected `by` after the `assert forall` statement.

---

### ✅ FIXED VERSION

```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self@.len() == bm@.len(),
    ensures
        self@.len() == ret@.len(),
        forall|i: int| #![auto] 0 <= i < ret@.len() ==>
            ret@[i] == (self@[i] || bm@[i]),
{
    let n: usize = self.bits.len();
    let mut i: usize = 0;
    let mut res_bits: Vec<u64> = Vec::new();
    let mut result = BitMap { bits: res_bits };
    while i < n
        invariant
            i <= n,
            n == self.bits@.len(),
            n == bm.bits@.len(),
            i == result.bits.len(),
            forall|k: int| #![auto] 0 <= k < i * 64 ==>
                result@[k] == (self@[k] || bm@[k]),
    {
        res_bits = result.bits;
        let u1: u64 = self.bits[i];
        let u2: u64 = bm.bits[i];
        let or_int: u64 = u1 | u2;
        proof {
            bit_or_64_proof(u1, u2, or_int);
            // Removed the problematic assert forall - not needed!
        }
        res_bits.push(or_int);
        result = BitMap { bits: res_bits };
        i = i + 1;
    }
    result
}
```

**Changes**:
1. ✅ Removed `decreases n - i` (not needed for this simple loop)
2. ✅ Simplified type casts in invariant
3. ✅ **Removed broken `assert forall` statement**
4. ✅ Cleaner postcondition formatting

---

## Detailed Change Analysis

### Change 1: Loop Invariant Simplification

| Aspect | Broken | Fixed | Impact |
|--------|--------|-------|--------|
| Type casts | `0 <= i as int <= n as int` | `i <= n` | Cleaner, Verus infers types |
| Bitmap length | `result.bits@.len() == i as int` | `i == result.bits.len()` | More idiomatic |
| Decreases clause | `decreases n - i` | (removed) | Verus auto-detects termination |

### Change 2: Proof Block Content

```diff
  proof {
      bit_or_64_proof(u1, u2, or_int);
-     assert forall|off: int| #![trigger result@[(i as int) * 64 + off]]
-         0 <= off && off < 64 ==>
-             result@[(i as int) * 64 + off]
-                 == (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off]);
  }
```

**Why this works**:
- The loop invariant already states: `forall|k: int| ... result@[k] == (self@[k] || bm@[k])`
- The `bit_or_64_proof` provides bit-level facts about the OR operation
- Verus can automatically connect these two facts without explicit assertion

### Change 3: Postcondition Formatting

```diff
  ensures
      self@.len() == ret@.len(),
-     forall|i: int| 0 <= i && i < self@.len() ==> ret@[i] == (self@[i] || bm@[i])
+     forall|i: int| #![auto] 0 <= i < ret@.len() ==>
+         ret@[i] == (self@[i] || bm@[i]),
```

**Improvements**:
- Added `#![auto]` trigger for better automation
- Used `ret@.len()` instead of `self@.len()` (more direct)
- Better formatting (multi-line)

---

## `set_bit()` Method - Both Correct

Both versions handle `set_bit()` correctly:

```rust
fn set_bit(&mut self, index: u32, bit: bool)
    requires
        index < old(self)@.len(),
    ensures
        self@ == old(self)@.update(index as int, bit),
{
    let seq_index: usize = (index / 64) as usize;
    let bit_index: u32 = index % 64;
    let bv_old: u64 = self.bits[seq_index];
    let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

    self.bits.set(seq_index, bv_new);
    proof {
        set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        assert_seqs_equal!(
            self.view(),
            old(self).view().update(index as int, bit)
        );
    }
}
```

This method was correctly generated and never had issues.

---

## `view()` Function - Minor Difference

### Broken Version
```rust
spec fn view(&self) -> Seq<bool> {
    let total_bits = self.bits@.len() * 64;
    Seq::new(total_bits, |i: int| {
        ((self.bits@[(i / 64) as int] >> ((i % 64) as nat)) & 0x1) == 1
    })
}
```

### Fixed Version
```rust
spec fn view(&self) -> Seq<bool> {
    let total_bits = self.bits@.len() * 64;
    Seq::new(total_bits, |i: int|
        get_bit64!(self.bits@[i / 64], (i % 64) as u64)
    )
}
```

**Difference**:
- Broken: Inline bit operations with explicit casts
- Fixed: Uses `get_bit64!` macro (cleaner, consistent with executable code)

Both are semantically equivalent, but the fixed version is more maintainable.

---

## What the Working Benchmark Shows

From `bitmap_2_expanded.rs`:

```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self@.len() == bm@.len(),
    ensures
        self@.len() == ret@.len(),
        forall|i: int| #![auto] 0 <= i < ret@.len() ==>
            (((0x1u64 & (ret.bits@[i / 64] >> ((i % 64) as u64))) == 1) ==
            (((0x1u64 & (self.bits@[i / 64] >> ((i % 64) as u64))) == 1) ||
             ((0x1u64 & (bm.bits@[i / 64] >> ((i % 64) as u64))) == 1))),
{
    // ... loop with simple proof block:
    proof {
        bit_or_64_proof(u1, u2, or_int);
    }
    // ... no assert forall needed!
}
```

**Key insight**: The reference implementation also uses a simple proof block without `assert forall`.

---

## Lessons for Proof Writing

### ✅ DO
- Keep proofs simple
- Let loop invariants do the work
- Trust proof functions (like `bit_or_64_proof`)
- Use `#![auto]` triggers when appropriate

### ❌ DON'T
- Over-specify with redundant `assert forall`
- Add complex assertions unless necessary
- Forget `by {}` clause when using `assert forall`
- Use `==>` when `implies` is clearer

---

## Testing Both Versions

### Broken Version
```bash
$ verus final_result_bitmap_todo.rs
error: expected `by`
   --> line 144:92
aborting due to 1 previous error
```

### Fixed Version
```bash
$ verus fixed_bitmap.rs
verification results:: 8 verified, 0 errors
```

---

## Summary

| Aspect | Analysis |
|--------|----------|
| **Root cause** | Missing `by {}` in `assert forall` |
| **Why it happened** | Proof generation over-engineered the proof |
| **Why repair failed** | Repair module lacked knowledge of `assert forall` syntax |
| **The fix** | Remove unnecessary assertion, simplify proof |
| **Time to fix** | 2 minutes (manual), 38 minutes (failed automation) |
| **Prevention** | Use simpler proof patterns, better syntax validation |

The fix demonstrates that **simpler is better** in formal verification - the cleanest proof is often no proof at all, just a call to the right lemma!
