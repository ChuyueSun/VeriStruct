# Loop Invariant Trigger Pattern Guide

**CRITICAL: This guide shows how to write loop invariants that avoid trigger errors in Verus.**

## The Trigger Error Problem

Verus triggers **cannot** have a variable appearing in both **arithmetic** and **non-arithmetic** positions.

### ❌ WRONG - Common Trigger Error

```rust
// ERROR: variable `i` in trigger cannot appear in both arithmetic and non-arithmetic positions
forall|i: int| 0 <= i < n ==>
    #[trigger] v@[i] == v1[length - 1 - i] &&  // ❌ i in both v@[i] (non-arith) and (length - 1 - i) (arith)
    #[trigger] v@[length - 1 - i] == v1[i]     // ❌ Same problem
```

**Why this fails:**
- `v@[i]` is non-arithmetic (array indexing)
- `length - 1 - i` is arithmetic (subtraction involving loop variable)
- Variable `i` appears in both contexts within the same trigger

### ✅ CORRECT - Solution: Don't trigger on arithmetic expressions

```rust
// GOOD: No triggers on expressions with arithmetic involving loop variables
forall|i: int| 0 <= i < n ==>
    v@[i] == v1[length as int - 1 - i] &&
    v@[length as int - 1 - i] == v1[i]
```

**Why this works:**
- We removed the `#[trigger]` annotations
- Verus will automatically choose appropriate triggers
- The invariant still expresses the same property

---

## Pattern 1: Vector Reverse

### Problem Context
When reversing a vector, we swap elements symmetrically:
- `v[i]` ←→ `v[length - 1 - i]`

### ❌ WRONG Invariant

```rust
for n in 0..(length / 2)
    invariant
        forall|i: int| 0 <= i < n ==>
            #[trigger] v@[i] == v1[length - 1 - i] &&  // ❌ ERROR!
            #[trigger] v@[length - 1 - i] == v1[i],    // ❌ ERROR!
```

### ✅ CORRECT Invariant

```rust
for n in 0..(length / 2)
    invariant
        length == v.len(),
        v.len() == v1.len(),
        0 <= n <= length / 2,
        // Swapped elements (already processed)
        forall|i: int| 0 <= i < n ==>
            v@[i] == v1[length as int - 1 - i] &&
            v@[length as int - 1 - i] == v1[i],
        // Unchanged elements (not yet processed)
        forall|i: int| n <= i < length - n ==>
            v@[i] == v1[i],
```

**Key points:**
1. No `#[trigger]` on expressions with `length - i`
2. Cast to `int` explicitly: `length as int - 1 - i`
3. Separate invariants for swapped vs unchanged elements

---

## Pattern 2: Swap Adjacent Pairs

### Problem Context
Swapping pairs: `(v[0], v[1])`, `(v[2], v[3])`, etc.

### ❌ WRONG Invariant

```rust
while i < length
    invariant
        forall|j: int| 0 <= j < i / 2 ==>
            #[trigger] v@[2 * j] == v1[2 * j + 1] &&  // ❌ ERROR if trigger used
```

### ✅ CORRECT Invariant

```rust
while i < length
    invariant
        length == v.len(),
        v.len() == v1.len(),
        i % 2 == 0,
        0 <= i <= length,
        // Pairs already swapped - NO triggers
        forall|j: int| 0 <= j < i / 2 ==>
            v@[2 * j] == v1[2 * j + 1] &&
            v@[2 * j + 1] == v1[2 * j],
        // Elements not yet processed
        forall|k: int| i <= k < length ==>
            v@[k] == v1[k],
```

---

## Pattern 3: General Permutation

When elements are rearranged but not in a simple swap pattern:

### ✅ CORRECT - Use spec function helper

```rust
spec fn mirror_index(len: int, i: int) -> int {
    len - 1 - i
}

// Then in invariant:
forall|i: int| 0 <= i < n ==>
    #[trigger] v@[i] == v1[mirror_index(length as int, i)]
```

**This works because:**
- The function call `mirror_index(...)` is non-arithmetic from trigger's perspective
- Arithmetic is hidden inside the spec function
- Verus can trigger on the function call

---

## Quick Rules

### ✅ DO:
1. **Remove triggers** from expressions with arithmetic involving loop variables
2. **Use separate foralls** for different parts of the invariant
3. **Cast explicitly**: `length as int - 1 - i`
4. **Use spec functions** to hide arithmetic from triggers

### ❌ DON'T:
1. **Never** put `#[trigger]` on `v@[n - i]` or similar arithmetic expressions
2. **Never** mix arithmetic and non-arithmetic uses of the same variable in a trigger
3. **Don't** assume triggers are always needed - often Verus picks them automatically

---

## Examples Summary

| Pattern | Key Technique |
|---------|--------------|
| Reverse | No triggers on `v@[length - 1 - i]` |
| Swap pairs | No triggers on `v@[2 * j]` or `v@[2 * j + 1]` |
| Permutation | Use spec function helper |
| General | Separate foralls, no triggers on arithmetic |

---

## Complete Reverse Example

```rust
fn reverse(v: &mut Vec<u64>)
    requires
        old(v)@.len() <= usize::MAX as nat,
    ensures
        v@.len() == old(v)@.len(),
        forall|i: int| 0 <= i < v@.len() ==>
            v@[i] == old(v)@[old(v)@.len() - 1 - i]
{
    let length = v.len();
    let ghost v1 = v@;
    for n in 0..(length / 2)
        invariant
            length == v.len(),
            v.len() == v1.len(),
            0 <= n <= length / 2,
            // Swapped - NO TRIGGERS
            forall|i: int| 0 <= i < n ==>
                v@[i] == v1[length as int - 1 - i] &&
                v@[length as int - 1 - i] == v1[i],
            // Unchanged
            forall|i: int| n <= i < length - n ==>
                v@[i] == v1[i],
    {
        let x = v[n];
        let y = v[length - 1 - n];
        v.set(n, y);
        v.set(length - 1 - n, x);
    }
}
```

---

## Remember

**The golden rule: If a quantifier involves arithmetic on a loop variable (like `n - i`, `2*i + 1`, etc.), DO NOT put a trigger on expressions using that variable.**

Let Verus choose triggers automatically, or use spec function helpers to hide the arithmetic.
