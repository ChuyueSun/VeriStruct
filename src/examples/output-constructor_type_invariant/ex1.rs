use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Prove type invariant holds before borrowing field
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
        proof {
            use_type_invariant(&*self); // Added by AI: get current invariant
            // Added by AI: Assert properties that must hold after helper returns
            // Added by AI: Since helper preserves buffer.len(), these still hold:
            assert(self.index < self.buffer.len());
        }
        helper(&mut self.buffer, self.index);
    }
}

}
```
