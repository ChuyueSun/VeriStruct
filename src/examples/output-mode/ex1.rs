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
            // REMOVED: ret.inv() - type_invariant automatically ensures this
    {
        Counter { count: 0 }
    }

    pub fn increment(&mut self)
        requires
            // REMOVED: old(self).inv() - type_invariant automatically requires this
        ensures
            // REMOVED: self.inv() - type_invariant automatically ensures this
            self.count == old(self).count + 1,
    {
        self.count += 1;
    }

    pub fn get(&self) -> (ret: usize)
        requires
            // REMOVED: self.inv() - type_invariant automatically requires this
        ensures
            ret == self.count,
    {
        self.count
    }
}

}
