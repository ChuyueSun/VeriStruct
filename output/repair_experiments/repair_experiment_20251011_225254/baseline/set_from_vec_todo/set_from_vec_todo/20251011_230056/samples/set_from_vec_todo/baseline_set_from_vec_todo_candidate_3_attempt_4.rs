verus! {

use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == set i | i in 0..self.vt.len() => self.vt@.index(i)
    {
        set i | i in 0..self.vt.len() => self.vt@.index(i)
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self)@ union set![v]
    {
        #[verifier::proof_block] {
            // No additional proof needed here
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v ∈ self@)
    {
        for i in 0..self.vt.len()
            invariant
                i <= self.vt.len(),
                // No element up to i matches v if we're still iterating
                forall |j: int| 0 <= j < i ==> self.vt@.index(j) != v
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
