# Verus View Function Guidelines

## 🚨 CRITICAL RULE: Check Tuple Size vs Field Count

**If the struct has N fields and the View type is an N-tuple, the view is TRIVIAL and MUST be refined!**

Examples:
  - ❌ TRIVIAL: `struct {ring, head, tail}` → `type V = (Seq<T>, nat, nat)` (3 fields, 3-tuple = NO abstraction)
  - ✅ GOOD: `struct {ring, head, tail}` → `type V = (Seq<T>, nat)` (3 fields, 2-tuple = ABSTRACTION!)
  - ✅ GOOD: `struct {data, len}` → `type V = Seq<T>` (2 fields, single type = ABSTRACTION!)

**Rule:** Tuple size MUST be STRICTLY LESS than field count to show true abstraction!

## View Refinement Guidelines
1. A good View abstraction should:
   - Represent the essential state of the data structure, not just copy its fields
   - Hide implementation details while preserving behavior
   - Be as simple as possible while being complete
   - **Have fewer elements in the tuple than fields in the struct** (or use a single non-tuple type)

2. Common refinements:
   - For collections (arrays, lists): Use Seq<T> instead of raw arrays
   - For indices: Use meaningful representations (e.g., range of valid elements)
   - For flag fields: Consider if they can be derived from other state
   - **Combine related fields into semantic abstractions** (e.g., ring+head+tail → active_elements)

3. Avoid redundancy:
   - Only include fields necessary for specification
   - Derive computable properties in method ensures clauses, not in the view
   - **Don't just wrap every field in a tuple - that's not abstraction!**

4. Prefer mathematical types over concrete types when possible
