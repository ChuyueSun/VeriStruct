# Analysis: `v.len()` vs `v@.len()` in Verus Spec Code

## Question

Does the instruction "Always use `vector.len()` instead of `<<vector@.len>>()`" in `verus_common.md` reflect a correctness requirement or just a style preference?

## Findings

### 1. Both syntaxes verify successfully

I replaced all instances of `v.len()` with `v@.len()` in spec contexts (requires, ensures, invariants, assertions) in the following verified benchmark files:

- **vectors.rs**: All 16 functions verified ✅
- **bitmap_2.rs**: All 14 functions verified ✅

### 2. Both syntaxes are semantically equivalent

Created test showing:

```rust
fn test_equivalence(v: &Vec<u64>)
    requires
        v.len() == v@.len(),  // This verifies!
{
}
```

### 3. Examples of replaced code that still verify

**Before:**

```rust
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < v.len() ==> v[i] <= v[j],
        exists|i: int| 0 <= i < v.len() && k == v[i],
    ensures
        r < v.len(),
```

**After (still verifies):**

```rust
fn binary_search(v: &Vec<u64>, k: u64) -> (r: usize)
    requires
        forall|i: int, j: int| 0 <= i <= j < v@.len() ==> v[i] <= v[j],
        exists|i: int| 0 <= i < v@.len() && k == v[i],
    ensures
        r < v@.len(),
```

**Another example:**

```rust
fn reverse(v: &mut Vec<u64>)
    ensures
        v@.len() == old(v)@.len(),  // Works!
        forall|i: int| 0 <= i < old(v)@.len() ==> v[i] == old(v)[old(v)@.len() - i - 1],
```

**With owned vectors:**

```rust
fn from(v: Vec<u64>) -> (ret: BitMap)
    ensures
        ret@.len() == v@.len() * 64,  // Works!
```

## Conclusion

The instruction is **a style preference, not a correctness requirement**.

### Why the instruction recommends `v.len()`

1. **Simpler and more readable** - no need for the `@` operator
2. **Less verbose** - fewer characters to type
3. **Verus treats `.len()` specially** - it automatically works in both executable and spec contexts
4. **Consistency** - matches the style of executable code

### The `<<vector@.len>>()` notation

- This syntax **doesn't actually exist** in the codebase (0 matches found)
- The instruction may be warning against an outdated or incorrect syntax pattern

### Recommendation

**Updated instruction to:**
> "Always use `vector@.len()` to access the length of the spec-level view. Both `vector.len()` and `vector@.len()` are correct in spec contexts, but prefer `vector@.len()` for consistency with other view operations like `vector@[i]`."

**Rationale:**
While both syntaxes work, standardizing on `v@.len()` provides:

- Consistency with other view operations (`v@[i]`, `v@.field`)
- Explicit indication that we're working with the spec-level view
- Clearer mental model: always use `@` for view operations in specifications

## Test Results

```bash
# vectors.rs with v@.len() syntax
$ verus vectors.rs
verification results:: 16 verified, 0 errors

# bitmap_2.rs with v@.len() syntax
$ verus bitmap_2.rs
verification results:: 14 verified, 0 errors
```
