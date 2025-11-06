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
            &&& ENSUREMENT3_1
            &&& ENSUREMENT3_2
        } else {
            &&& ENSUREMENT4_1
            &&& ENSUREMENT4_2
        }
        ...
```

## CRITICAL: old() Usage Rules - Immutable vs Mutable References

### ⚠️ RULE 1: NEVER use old() with Immutable References (&self, &T)

**For methods with `&self` parameter (immutable):**

**In `requires` clauses:**

- ✅ Use `self` directly - NO old() needed!
- ❌ NEVER use `old(self)` - this causes compilation errors!
- Example: `requires self.invariant()`

**In `ensures` clauses:**

- ✅ Use `self` directly - NO old() needed!
- ❌ NEVER use `old(self)` - not valid for immutable references
- Example: `ensures ret == self.some_property()`

**Common mistake to avoid:**

```rust
// ❌ WRONG - causes compilation error!
fn read_data(&self) -> T
    requires
        old(self).invariant(),  // ❌ ERROR: Cannot use old() on &self!
    ensures
        ret == old(self).value()  // ❌ ERROR: Cannot use old() on &self!
```

**Correct version:**

```rust
// ✅ CORRECT - use self directly
fn read_data(&self) -> T
    requires
        self.invariant(),  // ✅ Correct: Use self directly
    ensures
        ret == self.value()  // ✅ Correct: Use self directly
```

### ✅ RULE 2: ALWAYS use old() with Mutable References (&mut self, &mut T)

**For methods with `&mut self` parameter:**

**In `requires` clauses:**

- ✅ ONLY use `old(self)` - refers to the pre-state before the function executes
- ❌ NEVER use `self` - the post-state doesn't exist yet in preconditions
- Example: `requires parameter < old(self).spec_property()`

**In `ensures` clauses:**

- ✅ Use `self` - refers to the post-state after the function executes
- ✅ Use `old(self)` - refers to the pre-state for comparison
- Example: `ensures self.spec_property() == old(self).spec_property()`

**Common mistake to avoid:**

```rust
fn mutate_data(&mut self, param: ParamType)
    requires
        old(self).property() == self.property(),  // ❌ ERROR: Cannot use `self` in requires!
        param < self.property(),                   // ❌ ERROR: Cannot use `self` in requires!
```

**Correct version:**

```rust
fn mutate_data(&mut self, param: ParamType)
    requires
        param < old(self).property(),              // ✅ Correct: Only `old(self)` in requires
    ensures
        self.property() == old(self).property(),   // ✅ Correct: Can use both in ensures
```

### 📋 Quick Reference Table

| Parameter Type | requires clause | ensures clause |
|----------------|-----------------|----------------|
| `&self` (immutable) | ✅ `self.property()` | ✅ `ret == self.property()` |
| `&self` (immutable) | ❌ `old(self).property()` | ❌ `old(self).property()` |
| `&mut self` (mutable) | ✅ `old(self).property()` | ✅ `self.property()`, `old(self).property()` |
| `&mut self` (mutable) | ❌ `self.property()` | - |
| `&mut node` (mutable param) | ✅ `old(node).property()` | ✅ `node.property()`, `old(node).property()` |
| `&node` (immutable param) | ✅ `node.property()` | ✅ `node.property()` |

### 🎯 Simple Rule to Remember

```
IF parameter is &mut (mutable reference):
    USE old() in requires clause
ELSE IF parameter is & (immutable reference):
    DO NOT USE old() - use the parameter directly
```

## Return Value Naming

- When using the return value in an `ensures` clause, assign it a name if not already provided (change the return type of the function), e.g.:

```rust
fn func(arg) -> (retname: rettype)
```

- When using if-else blocks in ensures clauses, always use `&&&` instead of `&&` to connect multiple conditions, as shown in the example above.
