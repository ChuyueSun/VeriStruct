# Verus Match Syntax Guidelines

## Using `matches!` Macro

In Verus, the `matches!` macro must use Rust's standard macro syntax:

```rust
// CORRECT
pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    matches!(opt, MyOption::Some(_))
}

// INCORRECT - don't use this syntax
pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    matches opt {
        MyOption::Some(_) => true,
        MyOption::None => false,
    }
}
```

## Match with `arbitrary()` in Spec Functions

When writing spec functions that match on patterns with impossible/unreachable branches, use `arbitrary()` instead of `unreachable!()`:

```rust
// CORRECT
pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A
    recommends is_Some(opt)
{
    match opt {
        MyOption::Some(a) => a,
        MyOption::None => arbitrary(), // For unreachable branches in spec functions
    }
}

// INCORRECT
pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A
    recommends is_Some(opt)
{
    match opt {
        MyOption::Some(a) => a,
        MyOption::None => unreachable!(), // Don't use this in spec functions
    }
}
```

## Match in Executable Functions

For unreachable branches in executable functions, use `unreached()`:

```rust
pub fn unwrap(self) -> (a: A)
    requires
        is_Some(self),
    ensures
        a == get_Some_0(self),
{
    match self {
        MyOption::Some(a) => a,
        MyOption::None => unreached(), // For unreachable branches in exec functions
    }
}
```

## Match in Proof Functions

For unreachable branches in proof functions, use `proof_from_false()`:

```rust
pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
    requires
        is_Some(self),
    ensures
        a == get_Some_0(self),
{
    match self {
        MyOption::Some(a) => a,
        MyOption::None => proof_from_false(), // For unreachable branches in proof functions
    }
}
``` 