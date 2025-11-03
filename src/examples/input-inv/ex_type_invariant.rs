use vstd::prelude::*;

verus! {

pub struct Container {
    data: Vec<usize>,
    size: usize,
}

impl Container {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.size <= self.data.len()
        &&& self.data.len() > 0
    }

    pub fn get_size(&self) -> (ret: usize)
        requires
            self.inv(),  // ERROR: cannot call private function 'inv' with type_invariant
        ensures
            ret == self.size,
    {
        self.size
    }

    pub fn new(capacity: usize) -> (ret: Container)
        requires
            capacity > 0,
        ensures
            ret.inv(),  // ERROR: cannot call private function 'inv' with type_invariant
    {
        let mut data = Vec::new();
        let mut i = 0;
        while i < capacity
            invariant
                data.len() == i,
                i <= capacity,
        {
            data.push(0);
            i += 1;
        }
        Container {
            data,
            size: 0,
        }
    }

    pub fn update(&mut self, value: usize)
        requires
            old(self).inv(),  // ERROR: cannot call private function 'inv' with type_invariant
        ensures
            self.inv(),  // ERROR: cannot call private function 'inv' with type_invariant
            self.size == old(self).size + 1,
    {
        self.size += 1;
    }
}

}
