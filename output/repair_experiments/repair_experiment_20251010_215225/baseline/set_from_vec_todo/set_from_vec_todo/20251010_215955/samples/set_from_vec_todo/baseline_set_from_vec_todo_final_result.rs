use vstd::prelude::*;
use vstd::seq_lib::{group_seq_properties, set_of_seq};

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        set_of_seq(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self@).union(set![v]),
    {
        proof { }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall |j| 0 <= j < i ==> self.vt[j] != v,
        {
            if self.vt[i] == v {
                return true;
            }
        }
        false
    }
}

/* TSET CODE BELOW */

fn test(t: Vec<u64>)
{
    let mut vs: VecSet = VecSet::new();
    assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    assert(!contains2);
    let contains3 = vs.contains(3);
    assert(contains3);
    assert(vs@ =~= set![3, 5]);
}

pub fn main() {}

} // verus!

// BASELINE FINAL RESULT - set_from_vec_todo
// Total Attempts: 10
// Total Candidates: 4
// Final Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Success: False
// Completed: 2025-10-10T22:21:34.764865
