use vstd::prelude::*;

verus! {

/// A bounded counter that tracks a value and its maximum limit.
/// This demonstrates a View with a tuple type where we need to track
/// both the current value and the constraint (max_value).
pub struct BoundedCounter {
    value: u64,
    max_value: u64,
}

// ========== INFERRED VIEW IMPLEMENTATION ==========
impl View for BoundedCounter {
    type V = (nat, nat);  // (current_value, maximum_allowed)

    closed spec fn view(&self) -> Self::V {
        (self.value as nat, self.max_value as nat)
    }
}
// ==================================================

impl BoundedCounter {
    /// Creates a new counter with the given maximum value
    pub fn new(max: u64) -> (ret: Self)
    {
        BoundedCounter {
            value: 0,
            max_value: max,
        }
    }

    /// Increments the counter if not at maximum
    pub fn increment(&mut self) -> (success: bool)
    {
        if self.value < self.max_value {
            self.value = self.value + 1;
            true
        } else {
            false
        }
    }

    /// Returns the current value
    pub fn get(&self) -> (val: u64)
    {
        self.value
    }
}

} // verus!
