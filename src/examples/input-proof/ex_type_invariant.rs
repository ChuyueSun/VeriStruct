use vstd::prelude::*;
fn main() {}

verus! {

/// Example demonstrating use_type_invariant pattern
///
/// CRITICAL PATTERN: When struct has #[verifier::type_invariant]:
/// - MUST call use_type_invariant(&self) in proof blocks
/// - This establishes bounds and constraints from the invariant
/// - Without it, verifier can't prove arithmetic safety

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
                // TODO: add proof
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
            // TODO: add proof
        }
        self.value
    }
}

}
