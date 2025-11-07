You are an expert in Verus (a Rust-based verification framework). Your task is to replace every occurrence of `// TODO: add proof` or `// TODO: add invariants` with appropriate proof blocks or loop invariants that help Verus verify the program. If invariants already exist, reconsider the invariants and edit them if necessary.

⚠️ **CRITICAL #1**: For functions that update `Seq<T>` views (like `set_bit`, `set_element`), you MUST use the `assert_seqs_equal!` macro in your proof block. DO NOT try to prove sequence equality manually with `assert forall` - it will fail! See Section 2 for details.

⚠️ **CRITICAL #2**: When adding loop invariants, you MUST check Section 3's mandatory checklist and follow the patterns. Most verification failures come from missing bridge invariants or missing region invariants. DO NOT skip these checks.

Follow these guidelines carefully:

## Quickstart

- Seq<T> update: modify concrete state first, then in `proof { ... }` call `assert_seqs_equal!(self.view(), old(self).view().update(index as int, value))`.
- Loops with view()-based postconditions: add a processed-region bridge invariant, prove only the new segment each iteration, and include a `decreases` clause.
- Repeat relevant preconditions inside invariants (ordering, bounds, distinctness, etc.).
- Keep lengths stable; add explicit length-preservation ensures when mutating elements.
- Pair this guide with few-shots in `src/examples/input-proof/` ↔ `src/examples/output-proof/` (see `ex_chunk_bridge.rs`).

## Table of Contents

- 1. Core Rules
  - Seq<T> Update Operations (MOST IMPORTANT)
  - Length Preservation Postconditions
  - Proof Block Structure
- 2. Loop Invariants
  - Mandatory Checklist
  - Inherit Precondition Properties
  - Decreases Clauses for Loop Termination
  - Pattern: Recognizing When Bridge Invariants Are Needed
  - Strengthening Loop Invariants for Array Access
  - CRITICAL: Multi-Region Invariants for Two-Pointer Algorithms
  - Pattern: Chunked-to-Bit Bridging for Bitwise Loops
- 3. Other Proof Techniques
- 4. Common Proof Locations
- 5. Constraints
- 6. Verification

## 1. Core Rules

### Proof Block Structure

- For regular functions (`fn` or `pub fn`): Add proof blocks using the syntax `proof { ... }`
- For proof functions (`proof fn`): Write assertions directly in the function body - DO NOT use `proof { ... }` blocks
- Each proof block should be focused and minimal, containing only what's needed

### Seq<T> Update Operations (MOST IMPORTANT)

**⚠️ USE `assert_seqs_equal!` MACRO - DO NOT USE `assert forall`**

When a function updates a `Seq<T>` view (e.g., `set_bit`, `set_element`, `insert`, `update`), you MUST use the `assert_seqs_equal!` macro **AFTER the actual modification**:

```rust
// First do the actual modification
self.bits.set(seq_index, bv_new);

// Then prove it worked with assert_seqs_equal!
proof {
    assert_seqs_equal!(
        self.view(),
        old(self).view().update(index as int, value)
    );
}
```

**CRITICAL**: The `assert_seqs_equal!` macro must come AFTER the state modification, not before!

**Common mistakes to AVOID**:

- ❌ DON'T write: `assert forall|i: int| ...` (this will fail!)
- ❌ DON'T try to prove sequence equality manually
- ❌ DON'T skip this macro and leave proof block empty

**When to use this**:

- Any function that modifies exactly one position in a Seq-based view
- After calling operations like `self.data.set(...)` to update a single element
- When postcondition mentions `old(self)@.update(...)`
- When the function semantics are "change element at index i, keep rest unchanged"

**This macro automatically**:

- Proves sequence lengths match
- Proves element-wise equality with proper triggers
- Handles the connection between low-level field updates and high-level view updates

### Bit-Vector Macros and View Bridging (u64 chunks ↔ Seq<bool>)

When a data structure stores bits in `u64` chunks but exposes a `Seq<bool>` via `spec fn view(&self)`, follow these rules:

- Prefer `@` over `.view()` in proofs and invariants (e.g., `self@[i]`, `result@[k]`).
- Use the bit-vector lemmas for chunk-level correctness:
  - `set_bit64_proof(new, old, idx, bit)` after computing the updated `u64` value.
  - `bit_or_64_proof(u1, u2, or_int)` after computing `or_int = u1 | u2`.
- Use `get_bit64!` to connect a `u64` chunk to boolean bits in `Seq<bool>`:
  - Example: `get_bit64!(self.bits@[i / 64], (i % 64) as u64)`.
- For per-iteration proof of a newly produced chunk (e.g., in a loop), add a proof block:

```rust
proof {
    assert forall|off: int| 0 <= off && off < 64 ==>
        result@[(i as int) * 64 + off]
            == (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off])
    by {
        // justified by bit_or_64_proof(u1, u2, or_int)
    }
}
```

Syntax and arithmetic constraints:

- Use `==>` (not the word `implies`) in `assert forall`.
- Avoid chained inequalities. Rewrite `0 <= k < B` as `0 <= k && k < B`.
  - **CRITICAL**: Comparison operators (`<=`, `>=`, `<`, `>`, `==`, `!=`) are single tokens - NEVER split them with spaces!
  - ✅ Correct: `0 <= x && x <= n` (proper `<=` operator)
  - ❌ WRONG: `0 <= x && x < = n` (broken operator with space)
  - ✅ Correct: `a >= b && b > c` (proper operators)
  - ❌ WRONG: `a > = b && b > c` (broken operator with space)
- Parenthesize casts in products: `(i as int) * 64`.

### Length Preservation Postconditions

For mutation methods that modify array/vector elements but don't change the container size, explicitly add length preservation to postconditions:

```rust
fn update_element(&mut self, index: ElementIndex, value: ElementType)
    requires
        index < old(self)@.len(),
    ensures
        self@ == old(self)@.update(index as int, value),
        self.data@.len() == old(self).data@.len(),
```

Why: When mutation methods are called sequentially, Verus needs to track that container lengths remain unchanged. Without explicit postconditions, Verus cannot prove that subsequent calls satisfy their preconditions.

General pattern: For any `&mut self` method that (1) accesses elements via indices, (2) does NOT call `push`, `pop`, `resize`, etc., (3) only modifies element values → Always add `self.container@.len() == old(self).container@.len()`.

## 2. Loop Invariants

- Carefully review all existing lemmas defined in the file and invoke each one that is relevant to the current proof context, using the syntax `lemma_name(arg1, arg2, ...)`.
  - For example, if there are lemmas about sequence bounds or modular arithmetic, call them as needed, such as `lemma_mod_auto(self.vt.len() as int)`.
  - For lemmas about sequence properties, use the appropriate generic syntax, e.g., `broadcast use group_seq_properties`.
  - When reasoning about sequences or specifications, ensure that all applicable modular arithmetic and sequence-related lemmas from the file are called to support your proof.
- Use assertions strategically with `assert(condition)`
- When helpful, use the `by(...)` syntax for proof steps:
  - `by(nonlinear_arith)` for arithmetic reasoning
  - `by { ... }` for explicit proof steps

### Mandatory Checklist

1. ✓ Does the struct have a `spec fn view(&self)` or similar abstraction?
2. ✓ Is the postcondition expressed using `view()` or other spec functions?
3. ✓ Does the loop modify the concrete representation (fields, arrays)?
4. ✓ Are there function preconditions (sorted, non-negative, etc.)?
5. ✓ Does the loop process from multiple positions (two pointers, dual-end)?

**IF YES to questions 1-3:** You MUST add bridge invariants (see Section 3.2.2)
**IF YES to question 4:** You MUST inherit preconditions into invariants (see below)
**IF YES to question 5:** You MUST add invariants for ALL regions (see Section 3.3)

When adding loop invariants (marked by `// TODO: add invariants`), include:

- Identify and add invariants for EVERY variable that is READ in the loop:
  - For scalar variables (e.g., x, y)
  - For array/vector elements (e.g., x[k], v[i])
  - Include invariants about their initial values
- Identify and add invariants for EVERY variable that is WRITTEN in the loop:
  - For direct assignments (e.g., y = ...)
  - For vector/array updates (e.g., v.set(..., ...))
  - Repeat relevant invariants even if specified earlier
- Fully utilize spec functions and proof functions in the invariants

### Inherit Precondition Properties into Loop Invariants

When a loop's correctness depends on properties from the function's preconditions, those properties MUST be explicitly repeated in the loop invariants, even though they are "obviously" true from context.

**Common precondition properties to inherit:**

1. **Ordering properties**: `forall|i: int, j: int| 0 <= i <= j < array.len() ==> array[i] <= array[j]`
2. **Non-negative values**: `forall|i: int| 0 <= i < array.len() ==> array[i] >= 0`
3. **Distinct elements**: `forall|i: int, j: int| 0 <= i < j < array.len() ==> array[i] != array[j]`
4. **Bounded values**: `forall|i: int| 0 <= i < array.len() ==> array[i] < MAX_VALUE`
5. **Structural properties**: Any property about the structure of data that the algorithm relies on

**Abstract Pattern:**

```rust
fn algorithm(data: &DataStructure, target: ValueType) -> (result: ResultType)
    requires
        precondition_property_1(data),  // e.g., ordering, uniqueness, etc.
        precondition_property_2(data, target),
{
    // ... initialization ...
    while loop_condition
        invariant
            loop_bounds_invariant,
            precondition_property_1(data),  // ← MUST REPEAT preconditions!
            loop_correctness_invariant,
        decreases termination_metric
    {
        // loop body that relies on precondition_property_1
    }
}
```

**Why this matters**: Verus does not automatically assume preconditions remain true inside loops. Without explicitly stating these properties in invariants, Verus cannot use them to reason about why the loop maintains correctness (e.g., why narrowing ranges, partitioning data, or updating indices preserves desired properties).

### Decreases Clauses for Loop Termination

Every loop MUST have a `decreases` clause to prove termination:

```rust
while condition
    invariant
        // ... your invariants ...
    decreases expression  // ← REQUIRED
{
    // loop body
}
```

**Common patterns:**

1. **Incrementing counter** (`while i < n`):

   ```rust
   decreases n - i
   ```

2. **Decrementing counter** (`while i > 0`):

   ```rust
   decreases i
   ```

3. **Binary search / narrowing range** (`while i1 < i2`):

   ```rust
   decreases i2 - i1
   ```

4. **Narrowing range with != condition** (`while i1 != i2`):

   ```rust
   decreases i2 - i1  // Ensure i1 and i2 converge
   ```

5. **Complex expressions** - use the value that strictly decreases each iteration

**The decreases expression must:**

- Be non-negative (type `int` or `nat`)
- Strictly decrease on each loop iteration
- Prove the loop eventually terminates

**Key insight for narrowing range algorithms**: When maintaining a search range [i1, i2], ensure the invariant states that the target exists within the **current range** [i1, i2], not just somewhere in the entire collection. For example:

- ❌ Weak: `exists|i: int| 0 <= i < v.len() && v[i] == k`
- ✅ Strong: `exists|i: int| i1 <= i <= i2 && v[i] == k`

This ensures that when the loop exits with i1 == i2, the invariant directly proves the postcondition.

### Pattern: Recognizing When Bridge Invariants Are Needed

**Before writing loop invariants, check:**

1. Does the data structure have a `spec fn view(&self)` or similar abstraction function?
2. Is the postcondition expressed in terms of `view()` rather than raw fields?
3. Does the loop modify the underlying concrete representation?

**If YES to all three → You MUST add bridge invariants** (see Section 3.2.2 below)

### CRITICAL: Multi-Region Invariants for Two-Pointer Algorithms

When an algorithm processes an array/sequence from multiple positions (e.g., from both ends, or with multiple cursors), you MUST add separate invariants for EACH region.

**Pattern: Dual-end processing (moving from both ends toward middle)**

When processing indices from both ends (e.g., `left` moving right, `right` moving left):

```rust
for cursor in 0..midpoint
    invariant
        // Bounds
        0 <= cursor <= midpoint,
        cursor <= length - cursor,
        // Region 1: Left side [0, cursor) - already processed
        forall|i: int| 0 <= i < cursor ==>
            property_holds_for_processed(v[i], original[i]),
        // Region 2: Middle [cursor, length-cursor) - not yet touched
        forall|i: int| cursor <= i < length - cursor ==>
            v[i] == original[i],
        // Region 3: Right side [length-cursor, length) - already processed  ← CRITICAL!
        forall|i: int| length - cursor <= i < length ==>
            property_holds_for_processed(v[i], original[i]),
```

**Why all three regions matter:**

- When loop exits at `cursor = midpoint`
- Left covers `[0, midpoint)`
- Middle becomes `[midpoint, midpoint)` = **empty**
- Right covers `[midpoint, length)`
- Together: **full coverage** of `[0, length)`

**Common mistake**: Forgetting the third invariant for the right/back region. Without it, Verus has no information about what happened to elements processed from the other end, causing postcondition failures.

**Pattern: Multiple cursors/partitions**

For algorithms with multiple moving boundaries (e.g., partitioning, quicksort-style):

```rust
while condition
    invariant
        // All cursor positions and their relationships
        0 <= cursor1 <= cursor2 <= cursor3 <= length,
        // Region 1: [0, cursor1) - elements with property A
        forall|i: int| 0 <= i < cursor1 ==> has_property_A(v[i]),
        // Region 2: [cursor1, cursor2) - elements with property B
        forall|i: int| cursor1 <= i < cursor2 ==> has_property_B(v[i]),
        // Region 3: [cursor2, cursor3) - elements with property C
        forall|i: int| cursor2 <= i < cursor3 ==> has_property_C(v[i]),
        // Region 4: [cursor3, length) - unprocessed
        // (may not need invariant if no property required yet)
```

**General principle**: If your algorithm creates N distinct regions during execution, you typically need N-1 to N invariants describing what's true in each region.

### Strengthening Loop Invariants for Array Access

When loops access arrays/vectors using loop variables, Verus needs strong invariants to prove bounds safety:

1. **Track array lengths explicitly**: If accessing arrays/vectors using loop variables, add:

   ```rust
   n == self.data@.len(),
   n == other.data@.len(),
   ```

   where `n` is the loop bound. This helps Verus prove `i < array.len()` at each access.

2. **Add "bridge invariants" connecting concrete and abstract representations**:

**⚠️ CRITICAL PATTERN - Most common cause of verification failure!**

If the struct has `spec fn view(&self)` and the postcondition mentions `view()`, you MUST add TWO invariants:

   When a data structure has both:

- Concrete representation (e.g., `data: Vec<ChunkType>`)
- Abstract specification via `spec fn view(&self) -> Seq<ElementType>`

   You MUST add invariants at BOTH levels:

   **Raw level** (concrete):

   ```rust
   forall|j: int| 0 <= j < i ==>
       result.data@[j] == combine_chunks(self.data@[j], other.data@[j])
   ```

   **Spec level** (abstract) - **REQUIRED to prove postconditions about view()**:

   ```rust
   forall|k: int| 0 <= k < i * ITEMS_PER_CHUNK ==>
       extract_from_chunks(result.data@, k) ==
       combine_elements(
           extract_from_chunks(self.data@, k),
           extract_from_chunks(other.data@, k)
       )
   ```

   **Key insight**: The spec-level invariant should use the SAME EXPRESSION as the view() function definition. This creates a direct bridge from concrete state to abstract spec, allowing Verus to prove postconditions stated in terms of view().

   Without the spec-level invariant, Verus cannot connect loop progress to postconditions about view().

   **STEP-BY-STEP RECIPE (DO THIS EVERY TIME):**

   1. **Find** the `spec fn view(&self)` definition in the struct
   2. **Copy** the exact expression used inside `Seq::new(...)`
   3. **Add raw-level invariant** (about concrete fields):

      ```rust
      forall|j: int| 0 <= j < i ==> result.data@[j] == combine(self.data@[j], other.data@[j])
      ```

   4. **Add bridge invariant** (REQUIRED - copy the view() expression):

      ```rust
      forall|k: int| 0 <= k < i * CHUNK_SIZE ==>
          expression_from_view(result.data@, k) ==
          expected_result(expression_from_view(self.data@, k),
                         expression_from_view(other.data@, k))
      ```

   3. **Add proof blocks INSIDE loops**: After modifying data structures in a loop, add proof blocks to establish invariants for the new iteration:

   ```rust
   result = DataStructure { data: new_data };
   proof {
       assert forall|k: int| i * ITEMS_PER_CHUNK <= k < (i + 1) * ITEMS_PER_CHUNK implies
           result.view()[k] == expected_value(self.view()[k], other.view()[k])
       by {
           // Use relevant lemmas and properties here
       }
   }
   ```

### Pattern: Chunked-to-Bit Bridging for Bitwise Loops

When arrays/vectors store data in fixed-size chunks (e.g., machine words), but the `view()` exposes a per-element/bit `Seq<T>` (e.g., `Seq<bool>`), you must bridge from chunk-level updates to bit-level postconditions.

**Goal**: Prove a spec-level property for all elements/bits, while the loop processes one chunk per iteration.

**Invariants (before each iteration i):**

- 0 <= i <= chunks
- Result growth (if constructing a new buffer): `result_bits@.len() == i`
- Lengths are fixed: `self@.len() == n`, `other@.len() == n`
- Spec-level bridge for processed region:

  ```rust
  forall|k: int| #![auto]
      0 <= k < i * CHUNK_SIZE ==>
      result@[k] == combine(self@[k], other@[k])
  ```

**After producing the next chunk (at index i):**
Place a proof block that re-establishes only the new segment `[i*CHUNK_SIZE, (i+1)*CHUNK_SIZE)`:

```rust
proof {
    assert forall|b: int| 0 <= b < CHUNK_SIZE implies
        result@[i*CHUNK_SIZE + b] == combine(self@[i*CHUNK_SIZE + b], other@[i*CHUNK_SIZE + b])
    by {
        // Unfold the view() mapping between (chunk, bit) and flat index
        // Let r_chunk = result_chunks@[i];
        // Let a_chunk = self_chunks@[i];
        // Let b_chunk = other_chunks@[i];
        // Use a chunk-level lemma, e.g., `chunk_op_lemma(a_chunk, b_chunk, r_chunk)`
        // that shows element/bit b of r_chunk equals combine(bit b of a_chunk, bit b of b_chunk).
    }
}
```

**Tips**

- Split proof into two regions each iteration: processed-old `[0, i*CHUNK_SIZE)` carried by the invariant, plus new `[i*CHUNK_SIZE, (i+1)*CHUNK_SIZE)` proved in the by-block.
- Keep arithmetic in `int` for invariants and proofs; perform casts only at concrete operation sites.
- Add a `decreases` clause, e.g., `decreases chunks - i`.

**Postconditions** (example):

```rust
ensures
    ret@.len() == self@.len(),
    forall|k: int| #![auto] 0 <= k < ret@.len() ==> ret@[k] == combine(self@[k], other@[k])
```

**Common mistakes to avoid**

- Writing a single large `forall k < (i+1)*CHUNK_SIZE` without splitting; prove only the new segment each iteration.
- Mixing `nat` and `int` in indices; use `int` in specs, cast at the boundary.
- Placing the per-segment proof before the actual mutation; the proof must come after updating the concrete state.

#### Example template (pair with few-shot)

Use this as a minimal template and pair it with the few-shot example files in `src/examples/input-proof/ex_chunk_bridge.rs` and `src/examples/output-proof/ex_chunk_bridge.rs`:

```rust
let mut i: usize = 0;
while i < chunks
    invariant
        0 <= i as int <= chunks as int,
        out_chunks@.len() == i as int,
        a@.len() == b@.len() == len_bits as int,
        forall|k: int| #![auto]
            0 <= k < i as int * CHUNK_SIZE ==>
            view_from(out_chunks@, len_bits as int)[k] == combine(a@[k], b@[k]),
    decreases chunks as int - i as int
{
    let a_chunk = a.chunks[i];
    let b_chunk = b.chunks[i];
    let r_chunk = chunk_op(a_chunk, b_chunk);
    out_chunks.push(r_chunk);

    proof {
        assert forall|off: int| 0 <= off < CHUNK_SIZE implies
            view_from(out_chunks@, len_bits as int)[i as int * CHUNK_SIZE + off]
                == combine(a@[i as int * CHUNK_SIZE + off],
                           b@[i as int * CHUNK_SIZE + off])
        by { chunk_op_lemma(a_chunk, b_chunk, r_chunk, off); }
    }

    i += 1;
}
```

Notes:

- Keep names generic (`combine`, `chunk_op`, `chunk_op_lemma`, `CHUNK_SIZE`).
- Follow the order: concrete mutation → proof of the new segment.

## 3. Other Proof Techniques

**⚠️ CRITICAL: Type Invariant Usage**

When you see `#[verifier::type_invariant]` in the code, **EVERY** proof block in that impl block **MUST** start with `use_type_invariant(...)`:

**Syntax**:

```rust
// For &mut self methods (most common):
proof {
    use_type_invariant(&*self);  // ← Note the &* dereference!
    // Now invariant properties are available
}

// For &self methods:
proof {
    use_type_invariant(&self);  // ← No dereference needed
}
```

**Common errors if missing**:

- "possible arithmetic underflow/overflow"
- "possible division by zero"
- "precondition not satisfied" for array access
- "constructed value may fail to meet its declared type invariant"

**Pattern**: Always make this the **first line** in any proof block when type invariant exists.

- Carefully review all existing lemmas defined in the file and invoke each one that is relevant to the current proof context, using the syntax `lemma_name(arg1, arg2, ...)`.
  - For example, if there are lemmas about sequence bounds or modular arithmetic, call them as needed, such as `lemma_mod_auto(self.vt.len() as int)`.
  - For lemmas about sequence properties, use the appropriate generic syntax, e.g., `broadcast use group_seq_properties`.
  - When reasoning about sequences or specifications, ensure that all applicable modular arithmetic and sequence-related lemmas from the file are called to support your proof.
- Use assertions strategically with `assert(condition)`
- When helpful, use the `by(...)` syntax for proof steps:
  - `by(nonlinear_arith)` for arithmetic reasoning
  - `by { ... }` for explicit proof steps

## 4. COMMON PROOF LOCATIONS

- At function start
- Before loops
- At loop start
- At loop end
- Before key operations
- After key operations

## 6. CONSTRAINTS

- DO NOT modify any code outside of proof blocks, invariant declarations, or postconditions
- You MAY add postconditions to `ensures` clauses (e.g., length preservation)
- DO NOT change function signatures (parameters, return types), types, or preconditions
- DO NOT add new functions or types
- If no TODO markers exist, return code unchanged

## 7. VERIFICATION

- Ensure all proof blocks and invariants compile under Verus
- Remove all TODO placeholders

Return the ENTIRE file with your changes – not a diff or partial snippet.
