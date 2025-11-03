use vstd::prelude::*;

verus! {

/// A stack with a fixed maximum capacity.
/// This demonstrates a View with a tuple type tracking both
/// the stack contents and the maximum size constraint.
pub struct BoundedStack<T> {
    data: Vec<T>,
    max_capacity: usize,
}

impl<T> View for BoundedStack<T> {
    // TODO: implement this
    // HINT: Need to track both the stack contents and the max capacity
}

impl<T> BoundedStack<T> {
    /// Creates a new bounded stack with given capacity
    pub fn new(capacity: usize) -> (ret: Self)
        // TODO: add ensures
    {
        BoundedStack {
            data: Vec::new(),
            max_capacity: capacity,
        }
    }

    /// Pushes a value onto the stack if not full
    pub fn push(&mut self, value: T) -> (success: bool)
        // TODO: add ensures
    {
        if self.data.len() < self.max_capacity {
            self.data.push(value);
            true
        } else {
            false
        }
    }
}

} // verus!
