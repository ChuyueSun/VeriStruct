# Verus Set Usage Guide

## Overview
`Set<A>` is a specification type representing mathematical sets. Sets can be finite or infinite and are used primarily in specifications (spec functions, requires/ensures clauses).

## Construction

```rust
// Empty set
let s1 = Set::<A>::empty();

// Full set (all elements of type A)
let s2 = Set::<A>::full();

// Set from predicate
let s3 = Set::new(|x: nat| x < 10);

// Set literal using macro
let s4 = set![1, 2, 3, 4];
```

## Core Operations

```rust
// Check membership
s.contains(x)          // returns bool
s has x                // alternative syntax

// Insert/remove elements
s.insert(x)            // returns new set with x added
s.remove(x)            // returns new set with x removed

// Set operations
s1.union(s2)           // or s1 + s2
s1.intersect(s2)       // or s1 * s2
s1.difference(s2)      // or s1 - s2
s.complement()         // returns complement of s
s.filter(f)            // filter by predicate f

// Subset relation
s1.subset_of(s2)       // or s1 <= s2
```

## Finite Sets

```rust
// Check finiteness
s.finite()             // returns bool

// Operations on finite sets
s.len()                // cardinality (requires s.finite())
s.choose()             // picks arbitrary element

// Useful predicates
s.disjoint(s2)         // s and s2 have no common elements
```

## Equality

Use extensional equality `=~=` to compare sets:
```rust
ensures s1 =~= s2      // s1 and s2 contain same elements
```

## Common Axioms

Key broadcast axioms automatically available:
- `axiom_set_insert_same`: `s.insert(a).contains(a)`
- `axiom_set_remove_same`: `!s.remove(a).contains(a)`
- `axiom_set_union`: `s1.union(s2).contains(a) == (s1.contains(a) || s2.contains(a))`
- `axiom_set_ext_equal`: `s1 =~= s2 <==> forall|a| s1.contains(a) == s2.contains(a)`

Use `broadcast use group_set_axioms;` to enable all set axioms in a proof block.

## Example Usage

```rust
proof fn example_set_proof() {
    let s1 = set![1, 2, 3];
    let s2 = set![3, 4, 5];

    assert(s1.insert(4).contains(4));
    assert(s1.union(s2).contains(1));
    assert(s1.intersect(s2) =~= set![3]);
    assert(s1.finite() && s1.len() == 3);
}
```
