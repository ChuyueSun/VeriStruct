use vstd::prelude::*;

verus! {

pub struct VecWrapper {
    data: Vec<u64>,
}

impl VecWrapper {
    spec fn view(&self) -> Seq<u64> {
        self.data@
    }

    fn elementwise_or(&self, other: &VecWrapper) -> (ret: VecWrapper)
        requires
            self@.len() == other@.len(),
        ensures
            ret@.len() == self@.len(),
            forall|i: int| #![auto] 0 <= i < ret@.len() ==>
                ret@[i] == (self@[i] | other@[i]),
    {
        let n: usize = self.data.len();
        let mut i: usize = 0;
        let mut result_data: Vec<u64> = Vec::new();
        let mut result = VecWrapper { data: result_data };
        while i < n
        // TODO: add loop invariant
        {
            result_data = result.data;
            let val1: u64 = self.data[i];
            let val2: u64 = other.data[i];
            let combined: u64 = val1 | val2;
            // TODO: add proof
            result_data.push(combined);
            result = VecWrapper { data: result_data };
            i = i + 1;
        }
        result
    }
}

} // verus!
