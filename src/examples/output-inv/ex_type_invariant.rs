use vstd::prelude::*;

verus! {

pub struct Container {
    data: Vec<usize>,
    size: usize,
}

impl Container {
    // ========== INFERRED INVARIANT ==========
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.size <= self.data.len()
        &&& self.data.len() > 0
    }
    // ========================================

    pub fn get_size(&self) -> (ret: usize)
    {
        self.size
    }

    pub fn new(capacity: usize) -> (ret: Container)
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
    {
        self.size += 1;
    }
}

}
