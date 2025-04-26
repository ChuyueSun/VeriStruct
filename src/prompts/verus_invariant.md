# Verus Invariant Guidelines

1. An invariant is a property that must hold for all valid instances of the data structure

2. It should consider the relationships between fields and enforce structural properties

3. For collections, consider:
   - Size/capacity constraints
   - Range bounds for indices
   - Ordering properties
   - Non-nullity of critical components

4. Use `&&` for simple conjunctions and `&&&` for more complex ones

5. For circular data structures, think about how the head/tail pointers relate to each other

6. Include range checks when appropriate (e.g., indices must be less than length) 