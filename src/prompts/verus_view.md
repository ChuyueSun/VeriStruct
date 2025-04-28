# Verus View Function Guidelines

## View Refinement Guidelines
1. A good View abstraction should:
   - Represent the essential state of the data structure, not just copy its fields
   - Hide implementation details while preserving behavior
   - Be as simple as possible while being complete

2. Common refinements:
   - For collections (arrays, lists): Use Seq<T> instead of raw arrays
   - For indices: Use meaningful representations (e.g., range of valid elements)
   - For flag fields: Consider if they can be derived from other state

3. Avoid redundancy:
   - Only include fields necessary for specification
   - Derive computable properties in method ensures clauses, not in the view

4. Prefer mathematical types over concrete types when possible
