use vstd::prelude::*;

verus! {

/// A stack with a fixed maximum capacity.
/// This demonstrates a View with a tuple type tracking both
/// the stack contents and the maximum size constraint.
pub struct BoundedStack<T> {
    data: Vec<T>,
    max_capacity: usize,
}

// ========== INFERRED VIEW IMPLEMENTATION ==========
impl<T> View for BoundedStack<T> {
    type V = (Seq<T>, nat);  // (stack_contents, max_capacity)

    closed spec fn view(&self) -> Self::V {
        (self.data@, self.max_capacity as nat)
    }
}
// ==================================================

impl<T> BoundedStack<T> {
    /// Creates a new bounded stack with given capacity
    pub fn new(capacity: usize) -> (ret: Self)
    {
        BoundedStack {
            data: Vec::new(),
            max_capacity: capacity,
        }
    }

    /// Pushes a value onto the stack if not full
    pub fn push(&mut self, value: T) -> (success: bool)
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
