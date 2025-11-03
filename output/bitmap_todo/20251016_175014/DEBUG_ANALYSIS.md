# Bitmap Verification Failure - Debug Analysis

## Executive Summary

The bitmap benchmark failed with **4 verification errors** in 3 functions (`get_bit`, `set_bit`, and `or`). The root cause was an **incorrect view specification** that used `(nat, Set<nat>)` instead of `Seq<bool>`, making proofs unnecessarily complex.

**Result**: Fixed version verifies successfully (8 functions verified, 0 errors).

---

## Root Cause: Wrong View Specification

### Failed Version (Lines 94-105)
```rust
spec fn view(&self) -> (nat, Set<nat>) {
    let bits_seq = self.bits@;
    let len_bits = bits_seq.len() * 64;
    let set_bits = Set::new(|i: nat|
        i < len_bits && ({
            let chunk = bits_seq[( i / 64 ) as int];
            let offset = i % 64;
            ((chunk >> offset) & 0x1) == 1
        })
    );
    (len_bits, set_bits)
}
```

**Problem**: Returns a tuple of (length, set of indices). This makes reasoning about individual bit operations extremely difficult because:
- Need to prove set membership via bit arithmetic
- Set operations (insert/remove/union) don't naturally map to bit operations
- Verus SMT solver can't easily connect low-level bit manipulation to high-level set operations

### Ground Truth (Lines 77-82 in bitmap_2.rs)
```rust
spec fn view(&self) -> Seq<bool> {
    let total_bits = self.bits@.len() * 64;
    Seq::new(total_bits, |i: int|
        get_bit64!(self.bits@[i / 64], (i % 64) as u64)
    )
}
```

**Why it works**:
- Direct 1-to-1 mapping: each bit position maps to a boolean in the sequence
- Natural fit for update operations: `seq.update(i, value)`
- Built-in sequence reasoning in Verus with powerful lemmas

---

## Specific Missing/Broken Proof Blocks

### 1. `get_bit` Function (Line 138-148)

**Failed Version:**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        (index as nat) < self.view().0,
    ensures
        bit == self.view().1.contains(index as nat),  // ❌ Can't prove
{
    let seq_index: usize = (index / 64) as usize;
    let bit_index: u32 = index % 64;
    let bucket: u64 = self.bits[seq_index];
    get_bit64_macro!(bucket, bit_index as u64)
}
```

**Error**:
- Line 142: `bit == self.view().1.contains(index as nat)` - Postcondition fails
- The function returns the result of `get_bit64_macro!` but can't prove it equals checking set membership

**Ground Truth Fix:**
```rust
fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index < self@.len(),
    ensures
        bit == self@[index as int],  // ✅ Direct sequence indexing
{
    let seq_index: usize = (index / 64) as usize;
    let bit_index: u32 = index % 64;
    let bucket: u64 = self.bits[seq_index];
    get_bit64_macro!(bucket, bit_index as u64)
}
```

**Missing**: No proof block needed! With `Seq<bool>`, Verus can automatically prove the postcondition by unfolding the view definition.

---

### 2. `set_bit` Function (Lines 161-195)

**Failed Version:**
```rust
fn set_bit(&mut self, index: u32, bit: bool)
    requires
        (index as nat) < old(self).view().0,
    ensures
        self.view().0 == old(self).view().0,
        if bit {
            self.view().1 == old(self).view().1.insert(index as nat)  // ❌ Can't prove
        } else {
            self.view().1 == old(self).view().1.remove(index as nat)  // ❌ Can't prove
        },
{
    let seq_index: usize = (index / 64) as usize;
    let bit_index: u32 = index % 64;
    let bv_old: u64 = self.bits[seq_index];
    let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
    proof {
        set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
    }
    self.bits.set(seq_index, bv_new);
    proof {
        // ❌ BROKEN: This assert forall doesn't help at all
        assert forall|k: nat|
            k < old(self).view().0 ==> (
               (k == index as nat) ==> (
                  (#[trigger] self.view().1.contains(k) == bit)
                  && (old(self).view().1.contains(k) != bit)
               )
            ) by {
            // Empty body - no reasoning provided!
        };
    }
}
```

**Errors**:
- Lines 166-170: Postcondition about set insert/remove fails
- Lines 185-193: The `assert forall` is malformed and doesn't help

**Ground Truth Fix (Lines 106-129):**
```rust
fn set_bit(&mut self, index: u32, bit: bool)
    requires
        index < old(self)@.len(),
    ensures
        self@ == old(self)@.update(index as int, bit),  // ✅ Simple update
{
    let seq_index: usize = (index / 64) as usize;
    let bit_index: u32 = index % 64;
    let bv_old: u64 = self.bits[seq_index];
    let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
    proof {
        set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
    }
    self.bits.set(seq_index, bv_new);
    // ✅ KEY FIX: Use assert_seqs_equal! macro
    proof {
        assert_seqs_equal!(
            self.view(),
            old(self).view().update(index as int, bit)
        );
    }
}
```

**Missing Proof Block**:
- **`assert_seqs_equal!` macro** (lines 123-126)
- This powerful macro proves two sequences are equal by:
  1. Checking lengths match
  2. Checking each element matches
  3. Using triggers to instantiate the quantifiers properly
- Without this, Verus can't connect the bit-level change to the sequence-level update

---

### 3. `or` Function (Lines 208-247)

**Failed Version:**
```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self.view().0 == bm.view().0,
    ensures
        ret.view().0 == self.view().0,
        ret.view().1 == self.view().1.union(bm.view().1),  // ❌ Can't prove
{
    // ... loop code ...
    while i < n
        invariant
            0 <= i <= n,
            result.bits.len() == i,
            self.bits@.len() == n,
            bm.bits@.len() == n,
            // Only proves chunk-level OR
            forall|j: int| 0 <= j < i ==>
                result.bits@.index(j) == (self.bits@.index(j) | bm.bits@.index(j)),
        // ... loop body ...
    {
        // ...
    }
    proof {
        // ❌ EMPTY: Just a comment, no actual proof!
        // "That matches the union of sets at bit level."
    }
    result
}
```

**Error**:
- Line 213: Can't prove `ret.view().1 == self.view().1.union(bm.view().1)`
- The loop invariant only proves chunk-level OR, not bit-level union
- The proof block at lines 242-245 is **completely empty** (just a comment)

**Ground Truth Fix (Lines 132-171):**
```rust
fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self@.len() == bm@.len(),
    ensures
        self@.len() == ret@.len(),
        // ✅ Postcondition about individual bits, not sets
        forall|i: int| #![auto] 0 <= i < ret@.len() ==>
            get_bit64!(ret.bits@[i / 64], (i % 64) as u64) ==
            (get_bit64!(self.bits@[i / 64], (i % 64) as u64) ||
             get_bit64!(bm.bits@[i / 64], (i % 64) as u64)),
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
            // ✅ KEY: Invariant relates bits at bit-level
            forall|k: int| #![auto] 0 <= k < i * 64 ==>
                get_bit64!(result.bits@[k / 64], (k % 64) as u64) ==
                (get_bit64!(self.bits@[k / 64], (k % 64) as u64) ||
                 get_bit64!(bm.bits@[k / 64], (k % 64) as u64)),
    {
        res_bits = result.bits;
        let u1: u64 = self.bits[i];
        let u2: u64 = bm.bits[i];
        let or_int: u64 = u1 | u2;
        proof {
            bit_or_64_proof(u1, u2, or_int);
        }
        res_bits.push(or_int);
        result = BitMap { bits: res_bits };
        i = i + 1;
    }
    result  // ✅ No proof block needed!
}
```

**Missing**:
1. **Better invariant** (lines 153-156): The invariant should relate bits at the bit-level, not just chunks
2. **No proof block needed after loop**: With the right invariant and `Seq<bool>` view, the postcondition follows automatically

---

## Summary of Missing Proof Blocks

| Function | What's Missing | Why It's Needed |
|----------|----------------|-----------------|
| `get_bit` | Nothing (with Seq view) | Seq-based view makes postcondition trivial |
| `set_bit` | `assert_seqs_equal!` macro | Connects bit-level change to sequence update |
| `or` | Better invariant + nothing after loop | Proper invariant makes postcondition automatic |

---

## Key Lessons

1. **View specification matters**: The choice of abstraction (`Seq<bool>` vs `Set<nat>`) has huge impact on proof complexity
2. **Use vstd utilities**: `assert_seqs_equal!` is essential for proving sequence properties
3. **Invariants must match postconditions**: Loop invariants should be at the same level of abstraction as postconditions
4. **SMT solver limitations**: Set operations over bit arithmetic are too complex for automatic reasoning

---

## Verification Results

**Before Fix**: 5 verified, 3 errors (4 Verus errors total)
- ❌ `get_bit` postcondition failed
- ❌ `set_bit` postcondition failed
- ❌ `set_bit` assertion failed
- ❌ `or` postcondition failed

**After Fix**: 8 verified, 0 errors ✅
- ✅ All BitMap methods verify
- ✅ Test function verifies
- ✅ Helper lemmas verify
