use vstd::prelude::*;
use vstd::seq_lib::{group_seq_properties, seq_to_set};

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures |res: Set<u64>| res == seq_to_set(self.vt@)
    {
        seq_to_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures s@ == set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures self@ == old(self@).insert(v)
    {
        proof {
            assert(self@ == seq_to_set(self.vt@));
            assert(old(self@) == seq_to_set(old(self.vt@)));
        }
        self.vt.push(v);
        proof {
            assert(self.vt@ == old(self.vt@) + [v]);
            assert(seq_to_set(old(self.vt@) + [v]) == seq_to_set(old(self.vt@)) .union set![v]);
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained == self@.contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall k in 0..i { self.vt[k] != v }
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
