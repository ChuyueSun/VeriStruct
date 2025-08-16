# Verus Common Knowledge

## Important Notes
- ALWAYS use parentheses whenever possible for clarity!
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations.
- Keep top level docstrings at the top of the file, before `verus! {`. Do not place them after the `verus! {` declaration.
- Don't change any function signatures.

## Spec Functions
1. No Direct Method Calls:
   In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
   To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
   When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug.
5. Parentheses Usage:
   ALWAYS wrap conditions in parentheses, even for simple expressions. This makes precedence explicit and prevents errors.

## Operators
Verus extends Rust logical operators with low-precedence forms that are especially helpful in specification code:

Standard Operators: &&, ||, ==>, <==>
Low-Precedence Variants: &&& and |||

The meaning of &&& is the same as && (logical AND), and ||| is the same as || (logical OR), but with lower precedence. This allows you to write conditions in a "bulleted list" style that remains grouped in a logical manner:

```
&&& a ==> b
&&& c
&&& d <==> e && f
```

is equivalent to:

```
(a ==> b) && c && (d <==> (e && f))
```

Note:
- Implication (==>) and equivalence (<==>) bind more tightly than &&& and |||.
- Using &&&/||| can make long specifications clearer by grouping logical clauses neatly.
