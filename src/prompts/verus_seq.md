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

## Subrange Usage

1. Cast indices to `int`: `self.head as int`, not just `self.head`
2. Use `.subrange(start, end)`, not `seq[start..end]`
3. Wrap complex expressions: `(..).subrange((expression) as int, ...)`
4. For full length: `(sequence.len()) as int`
