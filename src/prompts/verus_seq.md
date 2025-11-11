# Verus Sequence Knowledge

Seq<T> is a mathematical sequence type used in specifications:

- Building: Seq::empty(), seq![x, y, z], Seq::singleton(x)
- Length: s.len()
- Indexing: s[i] (0-based)
- Subrange: s.subrange(lo, hi) gives elements from index lo (inclusive) to hi (exclusive)
- Concatenation: s1 + s2
- Update: s.update(i, v) returns a new sequence with index i updated to value v
- Contains: s.contains(v) checks if v is in the sequence
- Push/pop: s.push(v), s.pop() (returns new sequence, doesn't modify original)
- Equality: Use `=~=` for sequence equality, NOT `==` (e.g., `seq1 =~= seq2`, `self@.0 =~= old(self)@.0.subrange(1, (old(self)@.0.len()) as int)`)

You can use forall or exists for properties over sequences.

## CRITICAL: Postconditions for Seq<T> Update Operations

**For functions that update a single element in a sequence-based view**:

**✅ PREFER** - Use `.update()` for succinct, provable specifications:

```rust
fn update_element(&mut self, idx: usize, value: T)
    requires
        idx < old(self)@.len(),
    ensures
        self@ == old(self)@.update(idx as int, value),  // ✅ Succinct!
```

**❌ AVOID** - Verbose element-wise specifications (makes proofs much harder):

```rust
ensures
    self@.len() == old(self)@.len(),
    forall|i: int| 0 <= i && i < self@.len() && i != idx as int ==> self@[i] == old(self)@[i],
    self@[idx as int] == value,
```

**Why `.update()` is better**:

1. More concise and readable
2. Directly matches proof patterns (pairs with `assert_seqs_equal!`)
3. Easier for Verus SMT solver to reason about
4. Standard pattern in Verus for sequence modifications

**When to use this pattern**:

- Any function that modifies exactly one position in a Seq-based view
- After operations that update a single element (e.g., `self.data.set(index, value)`)
- Functions with postconditions about changing one element while preserving others

## Subrange Usage

1. Cast indices to `int`: `self.head as int`, not just `self.head`
2. Use `.subrange(start, end)`, not `seq[start..end]`
3. Wrap complex expressions: `(..).subrange((expression) as int, ...)`
4. For full length: `(sequence.len()) as int`
