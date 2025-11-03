use vstd::prelude::*;

verus! {

pub struct Buffer {
    data: Vec<usize>,
}

impl Buffer {
    pub open spec fn is_sorted(&self) -> bool {
        forall|i: int, j: int| 0 <= i < j < self.data.len() ==> self.data[i] <= self.data[j]
    }

    pub fn add_element(&mut self, val: usize)
        requires
            old(self).is_sorted(),
        ensures
            self.data.len() == old(self).data.len() + 1,
    {
        proof {
            // FIXED: Use assertions with spec expressions instead of calling methods
            assert(old(self).is_sorted() ==>
                val >= old(self).data[old(self).data.len() - 1]);
        }
        self.data.push(val);
    }
}

}
