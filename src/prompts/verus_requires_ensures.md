# Verus Requires and Ensures Guidelines

## Formatting for `requires` and `ensures`

```rust
fn func(arg) -> rettype
    requires
        REQUIREMENT1,
        REQUIREMENT2,
        ...
    ensures
        ENSUREMENT1,
        ENSUREMENT2,
        if COND {
        ENSUREMENT3
        } else {
        ENSUREMENT4
        }
        ...
```

- In requires, use `old(self)` to refer to the pre-state of an &mut variable.
- When using the return value in an `ensures` clause, assign it a name if not already provided (change the return type of the function), e.g.:

```rust
fn func(arg) -> (retname: rettype)
``` 