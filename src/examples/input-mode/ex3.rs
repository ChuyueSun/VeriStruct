use vstd::prelude::*;

verus! {

pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    // ERROR: pub spec function must be marked as 'open' or 'closed'
    pub spec fn size(&self) -> nat {
        self.items.len()
    }

    // ERROR: pub spec function must be marked as 'open' or 'closed'
    pub spec fn is_empty(&self) -> bool {
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
