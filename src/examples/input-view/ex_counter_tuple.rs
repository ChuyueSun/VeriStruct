use vstd::prelude::*;

verus! {

/// A bounded counter that tracks a value and its maximum limit.
/// This demonstrates a View with a tuple type where we need to track
/// both the current value and the constraint (max_value).
pub struct BoundedCounter {
    value: u64,
    max_value: u64,
}

impl View for BoundedCounter {
    // TODO: implement this
    // HINT: Test code uses counter@.0 for current value and counter@.1 for max
}

impl BoundedCounter {
    /// Creates a new counter with the given maximum value
    pub fn new(max: u64) -> (ret: Self)
        // TODO: add ensures
    {
        BoundedCounter {
            value: 0,
            max_value: max,
        }
    }

    /// Increments the counter if not at maximum
    pub fn increment(&mut self) -> (success: bool)
        // TODO: add requires and ensures
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
        // TODO: add ensures
    {
        self.value
    }
}

} // verus!

// Test code that uses the View
#[cfg(test)]
mod tests {
    use super::*;

    fn test_counter() {
        let mut counter = BoundedCounter::new(10);
        // These assertions would use the View's tuple structure:
        // assert(counter@.0 == 0);        // current value
        // assert(counter@.1 == 10);       // max value

        let success = counter.increment();
        // assert(success);
        // assert(counter@.0 == 1);        // value incremented
        // assert(counter@.1 == 10);       // max unchanged
    }
}
