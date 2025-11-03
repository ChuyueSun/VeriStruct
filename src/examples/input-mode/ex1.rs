use vstd::prelude::*;

verus! {

pub struct Counter {
    count: usize,
}

impl Counter {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        self.count < 100
    }

    pub fn new() -> (ret: Counter)
        ensures
            ret.inv(),  // ERROR: cannot call inv() explicitly with type_invariant
    {
        Counter { count: 0 }
    }

    pub fn increment(&mut self)
        requires
            old(self).inv(),  // ERROR: cannot call inv() explicitly with type_invariant
        ensures
            self.inv(),  // ERROR: cannot call inv() explicitly with type_invariant
            self.count == old(self).count + 1,
    {
        self.count += 1;
    }

    pub fn get(&self) -> (ret: usize)
        requires
            self.inv(),  // ERROR: cannot call inv() explicitly with type_invariant
        ensures
            ret == self.count,
    {
        self.count
    }
}

}
