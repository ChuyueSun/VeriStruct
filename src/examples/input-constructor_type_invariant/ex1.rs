Failed assertion
```
Line 32-32:
            helper(&mut self.buffer, self.index);
Error: constructed value may fail to meet its declared type invariant
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Type invariant reconstruction after mutable field borrow
fn helper<T>(vec: &mut Vec<T>, index: usize)
    ensures
        vec@.len() == old(vec)@.len(),
{}

struct Container<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> Container<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.buffer.len() > 0
        &&& self.index < self.buffer.len()  // Depends on buffer.len()!
    }

    pub fn modify(&mut self)
    {
        // Error: Verus can't prove type invariant holds after helper returns
        // because index < buffer.len() depends on buffer.len() being unchanged
        helper(&mut self.buffer, self.index);
    }
}

}
```
