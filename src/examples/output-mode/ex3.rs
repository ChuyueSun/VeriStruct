use vstd::prelude::*;

verus! {

pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    // FIXED: Added 'open' - implementation is public and visible
    pub open spec fn size(&self) -> nat {
        self.items.len()
    }

    // FIXED: Added 'open' - implementation is public and visible
    pub open spec fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    pub fn new() -> (ret: Stack<T>)
        ensures
            ret.is_empty(),
    {
        Stack { items: Vec::new() }
    }
}

}
