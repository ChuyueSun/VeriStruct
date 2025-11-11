# Few-Shot Example Patterns for Spec Inference and Proof Generation

## Purpose

These examples teach the LLM **patterns** for common verification scenarios, not specific implementations. They show:

1. **Use `@` notation** instead of `.view()`
2. **Loop invariants** that connect concrete and abstract levels
3. **Simple proof blocks** that call proof functions
4. **Common spec patterns** (getter, setter, constructor, etc.)

---

## Key Patterns

### Pattern 1: Use @ Shorthand for View

**DON'T**:

```rust
requires
    index < self.view().len()
ensures
    ret.view().len() == old(self).view().len()
```

**DO**:

```rust
requires
    index < self@.len()
ensures
    ret@.len() == old(self)@.len()
```

**Taught by**: `ex_bitmap.rs` (input/output-requires)

---

### Pattern 2: Setter Uses .update()

**DON'T**:

```rust
ensures
    self@.len() == old(self)@.len(),
    forall|i: int| i != index ==> self@[i] == old(self)@[i],
    self@[index] == value,
```

**DO**:

```rust
ensures
    self@ == old(self)@.update(index as int, value),
```

**Taught by**: `ex_bitmap.rs` (output-requires, update_element method)

---

### Pattern 3: Loop Invariants Must Connect Levels

**DON'T** (incomplete):

```rust
while i < n
    invariant
        i <= n,
        self@.len() == other@.len(),  // Abstract level only!
```

**DO** (complete):

```rust
while i < n
    invariant
        i <= n,
        n == self.items@.len(),     // ← Connect to concrete!
        n == other.items@.len(),     // ← Connect to concrete!
        i == result.items.len(),
        forall|k: int| #![auto] 0 <= k < result@.len() ==>
            result@[k] == combine_operation(self@[k], other@[k]),
```

**Taught by**: `ex_bitmap_loop.rs` (output-proof)

**Why**: Without `n == self.items@.len()`, Verus can't prove `i < n` implies `i < self.items.len()`, causing precondition failures on `self.items[i]`.

---

### Pattern 4: Simple Proof Blocks

**DON'T** (over-engineered):

```rust
proof {
    lemma_function(args);
    assert forall|x| condition ==> conclusion by {
        // Complex nested proofs
        assert(property1);
        assert(property2);
        // ...
    }
    assert_seqs_equal!(seq1, seq2);
}
```

**DO** (minimalist):

```rust
proof {
    lemma_function(args);
    // That's it - let the invariant do the work
}
```

**Taught by**: `ex_bitmap_loop.rs` (output-proof, proof block in loop body)

**Why**: Simpler proofs are more reliable. The loop invariant already states the property we need.

---

### Pattern 5: Avoid Empty `by {}` Clauses

**DON'T**:

```rust
assert forall|x| P(x) ==> Q(x) by {
    // Empty - Verus won't be able to prove this!
};
```

**DO** (Option A - Preferred):

```rust
// Just don't use assert forall if you have nothing to say
proof {
    lemma_function(args);
}
```

**DO** (Option B - If really needed):

```rust
assert forall|x| P(x) implies Q(x) by {
    lemma_that_helps(x);
    assert(specific_fact(x));
}
```

**Taught by**: `ex_bitmap_loop.rs` (shows simple proof without assert forall)

---

## Example File Structure

### For spec_inference (requires/ensures)

**Input**: `input-requires/ex_*.rs`

- Shows code with `// TODO: add requires and ensures`
- Uses generic names (DataStructure, Container, ItemType, etc.)
- Demonstrates common patterns (getter, setter, constructor, etc.)

**Output**: `output-requires/ex_*.rs`

- Shows completed specs using `@` notation
- Demonstrates correct patterns
- Includes explanatory comments

### For proof_generation (loop invariants and proofs)

**Input**: `input-proof/ex_*.rs`

- Shows code with `// TODO: add loop invariant` and `// TODO: add proof`
- Uses generic names
- Demonstrates common loop patterns

**Output**: `output-proof/ex_*.rs`

- Shows complete loop invariants with concrete/abstract connections
- Shows simple proof blocks
- Includes explanatory comments about critical patterns

---

## How LLM Uses These

### During spec_inference

1. LLM sees input example with TODOs
2. LLM sees output example with completed specs using `@`
3. LLM learns: "Use `@` not `.view()`", "Use `.update()` for setters"
4. LLM applies pattern to actual code

### During proof_generation

1. LLM sees input with TODO markers in loops
2. LLM sees output with complete invariants connecting `n == vec.len()`
3. LLM learns: "Add `n == container@.len()` facts", "Keep proofs simple"
4. LLM applies pattern to actual code

---

## Benefits of This Approach

✅ **Generic**: Patterns apply to many data structures
✅ **Educational**: Clear comments explain WHY
✅ **Correct**: Based on working verified code
✅ **Focused**: Each example teaches specific patterns
✅ **Maintainable**: Easy to add new patterns

---

## Adding New Examples

To add a new pattern:

1. **Create input file**: `input-{stage}/ex_{pattern_name}.rs`
   - Use generic names (Container, ItemType, etc.)
   - Include TODOs where pattern applies
   - Keep it focused on ONE pattern

2. **Create output file**: `output-{stage}/ex_{pattern_name}.rs`
   - Show correct implementation
   - Add comments explaining critical parts
   - Use `@` notation consistently

3. **Document pattern**: Add to this README
   - What problem it solves
   - DON'T vs DO examples
   - Why it matters

---

## Current Examples

### spec_inference (requires/ensures)

| Example | Pattern | Key Teaching |
|---------|---------|--------------|
| `ex_bitmap.rs` | Data structures with view | Use `@` not `.view()` |
| `ex_seq.rs` | Sequence operations | Use `=~=` for seq equality |
| `ex1.rs` | Complex data structure | Comprehensive specification |

### proof_generation (invariants/proofs)

| Example | Pattern | Key Teaching |
|---------|---------|--------------|
| `ex_bitmap_loop.rs` | Vector iteration with combine | Connect `n == vec@.len()` in invariants |
| | | Keep proof blocks simple |

---

## Impact on bitmap_todo

### Before Examples

Original spec_inference output used:

```rust
self.view().len()           // Verbose ❌
old(self).view().field      // Verbose ❌
ret.view()[index]           // Verbose ❌
```

### With Examples

Should generate:

```rust
self@.len()                 // Clean ✅
old(self)@.field            // Clean ✅
ret@[index]                 // Clean ✅
```

### Loop Invariant Improvement

Before:

```rust
invariant
    self@.len() == other@.len(),  // Missing connection! ❌
```

After (with examples):

```rust
invariant
    n == self.items@.len(),       // Connected! ✅
    n == other.items@.len(),       // Connected! ✅
```

---

## Expected Improvements

With these examples, spec_inference and proof_generation should:

✅ Generate cleaner code with `@` notation
✅ Include `n == vec@.len()` in loop invariants
✅ Avoid unnecessary `assert forall` statements
✅ Keep proof blocks simple

**Result**: Should improve from 7/8 to 8/8 verified functions!
