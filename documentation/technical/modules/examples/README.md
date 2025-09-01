# VerusAgent Example Documentation

## Overview

This directory contains detailed examples showing how VerusAgent modules process different types of data structures and verification challenges.

## Examples

### 1. [RingBuffer](rb_type_invariant.md)
A circular buffer implementation demonstrating:
- Sequence abstraction
- Wrap-around operations
- Capacity management
- Index bounds verification

### 2. [BitMap](bitmap.md)
A bit vector implementation showing:
- Bit-level operations
- Mathematical mapping
- Macro integration
- Operation proofs

## Comparison Matrix

| Feature | RingBuffer | BitMap |
|---------|------------|--------|
| Abstraction | `(Seq<T>, usize)` | `Seq<bool>` |
| Operations | Sequence manipulation | Bit manipulation |
| Invariants | Structural bounds | Bit operation proofs |
| Proofs | State transitions | Operation correctness |
| Complexity | Wrap-around logic | Bit-level mapping |

## Module Processing

### View Inference
- RingBuffer: Sequence + capacity abstraction
- BitMap: Boolean sequence abstraction

### View Refinement
- RingBuffer: Maintains dual representation
- BitMap: Uses flat boolean sequence

### Invariant Inference
- RingBuffer: Explicit structural invariants
- BitMap: Relies on Vec invariants

### Specification Inference
- RingBuffer: State transition specs
- BitMap: Bit operation specs

### Proof Generation
- RingBuffer: State consistency proofs
- BitMap: Operation correctness proofs

## Verification Patterns

### 1. State Management
```rust
// RingBuffer: State transitions
ensures
    self@.1 == old(self)@.1,  // Capacity preserved
    self@.0.len() == old(self)@.0.len() + 1  // Length updated

// BitMap: State updates
ensures
    self@ == old(self)@.update(index as int, bit)  // Bit updated
```

### 2. Operation Verification
```rust
// RingBuffer: Sequence operations
proof {
    use_type_invariant(&*self);
    lemma_mod_auto(self@.1 as int);
}

// BitMap: Bit operations
proof {
    set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
}
```

### 3. Abstraction Mapping
```rust
// RingBuffer: Wrap-around handling
if self.tail >= self.head {
    ((self.ring)@.subrange(self.head as int, self.tail as int), cap)
} else {
    ((self.ring)@.subrange(self.head as int, cap as int)
        .add((self.ring)@.subrange(0, self.tail as int)), cap)
}

// BitMap: Bit-level mapping
Seq::new(total_bits, |i: int|
    get_bit64!(self.bits@[i / 64], (i % 64) as u64)
)
```

## Common Verification Challenges

1. Bounds Management:
   - RingBuffer: Wrap-around indices
   - BitMap: Bit-level indices

2. State Preservation:
   - RingBuffer: Capacity and elements
   - BitMap: Bit vector contents

3. Operation Correctness:
   - RingBuffer: Sequence operations
   - BitMap: Bit manipulations

4. Abstraction Maintenance:
   - RingBuffer: Dual view consistency
   - BitMap: Bit-to-boolean mapping

## Best Practices

1. View Selection:
   - Choose appropriate mathematical types
   - Maintain minimal representation
   - Handle special operations
   - Preserve semantics

2. Invariant Design:
   - Focus on essential properties
   - Use appropriate proof mechanisms
   - Maintain consistency
   - Handle edge cases

3. Specification Style:
   - Clear operation contracts
   - Precise state updates
   - Comprehensive coverage
   - Efficient verification

4. Proof Structure:
   - Targeted assertions
   - Appropriate lemmas
   - Operation verification
   - State consistency

## Conclusion

These examples demonstrate how VerusAgent modules adapt to different verification challenges:

1. Abstraction Level:
   - High-level sequence operations
   - Low-level bit manipulations

2. Proof Techniques:
   - State transition proofs
   - Operation correctness proofs

3. Specification Styles:
   - Sequence-based contracts
   - Bit-level contracts

4. Verification Approaches:
   - Structural verification
   - Operational verification
