use vstd::prelude::*;

verus! {

/// Example demonstrating use_type_invariant pattern
///
/// CRITICAL PATTERN: When struct has #[verifier::type_invariant]:
/// - MUST call use_type_invariant(&self) in proof blocks
/// - This establishes bounds and constraints from the invariant
/// - Without it, verifier can't prove arithmetic safety
///
/// ✅ CORRECT: use_type_invariant(&self) in every proof block
/// ❌ WRONG: Empty proof blocks with type invariant (will fail verification)

pub struct BoundedCounter {
    value: usize,
    max: usize,
}

impl BoundedCounter {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.value <= self.max
        &&& self.max > 0
    }

    pub fn new(maximum: usize) -> (ret: Self)
        requires maximum > 0
    {
        BoundedCounter { value: 0, max: maximum }
    }

    pub fn increment(&mut self) -> (success: bool)
    {
        if self.value < self.max {
            proof {
                // ✅ CRITICAL: For &mut self methods, use &*self (dereference first)
                use_type_invariant(&*self);
                // This establishes: self.value <= self.max, self.max > 0
                // Without it, verifier can't prove self.value + 1 won't overflow!

                // The invariant proves:
                // - self.value <= self.max (so value < max is valid)
                // - self.max > 0 (so operations won't divide by zero)
                // - Addition won't overflow because value < max
            }
            self.value = self.value + 1;
            true
        } else {
            false
        }
    }

    pub fn get(&self) -> (val: usize)
    {
        proof {
            // ✅ For &self methods, use &self directly
            use_type_invariant(&self);
            // This ensures the type invariant holds at function entry
        }
        self.value
    }
}

}

fn main() {}
