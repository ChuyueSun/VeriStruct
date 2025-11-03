verus! {

use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // We collect all elements of self.vt into a set
        set x | exists i: int. 0 <= i < self.vt@.len() && self.vt@[i] == x
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self@ == old(self@) union set![v],
    {
        proof {
            let old_vt = self.vt@;
            self.vt.push(v);
            assert(self.vt@ == old_vt + seq![v]);
            assert(self.view() == old(self.view()).union(set![v]));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self@.contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                forall k: int :: 0 <= k < i ==> self.vt@[k] != v,
            decreases self.vt.len() - i
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
