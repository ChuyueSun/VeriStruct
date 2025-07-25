use vstd::prelude::*;
// use vstd::view::View;

pub fn main() {}

verus! {
    pub open spec fn binary_search_spec(v: &Vec<u64>, k: u64) -> (r: nat)
    {
        let mut i1: usize = 0;
        let mut i2: usize = v.len() - 1;
        while i1 != i2
        {
            let ix = i1 + (i2 - i1) / 2;
            if v[ix] < k {
                i1 = ix + 1;
            } else {
                i2 = ix;
            }
        }
        i1
    }

    pub proof fn bin_search_properties(v: &Vec<u64>, k: u64) 
        requires 
            v@.len() > 0, // function assumes vector is non-empty
            forall |i: nat| i + 1 < v@.len() => v@[i] <= v@[i + 1] // binary search only works for sorted vectors 
        ensures
            0 <= binary_search_spec(v, k) <= v@.len(), // must be a valid index or v.len() (larger than all)
            exists |i: nat| i < v@.len() && v@[i] == k => v@[binary_search_spec(v, k)] == k, // if found, returned idx must have right value
            (forall |i: nat| i < v@.len() => k > v@[i]) => binary_search_spec(v, k) == v@.len(), // if larger than all, returned idx must be v.len()
    {
    }
}