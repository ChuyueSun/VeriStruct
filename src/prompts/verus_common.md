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

## Proof Blocks - CRITICAL SYNTAX RULES

**🚫 NEVER use executable control flow (if/else/match) inside `proof { }` blocks!**

Proof blocks are spec-level contexts. They can only contain:
- `assert(...)` statements
- `assume(...)` statements
- Lemma/proof function calls
- Variable bindings with spec expressions

❌ **WRONG - Executable if/else in proof:**
```rust
proof {
    if condition { assert(x); } else { assert(y); }  // SYNTAX ERROR!
}
```

✅ **CORRECT - Use implication instead:**
```rust
proof {
    assert(condition ==> x);
    assert(!condition ==> y);
}
```

❌ **WRONG - Executable match in proof:**
```rust
proof {
    match opt { Some(v) => assert(v > 0), None => {} }  // SYNTAX ERROR!
}
```

✅ **CORRECT - Use implication or spec-level reasoning:**
```rust
proof {
    assert(opt.is_Some() ==> opt.unwrap() > 0);
}
```

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
