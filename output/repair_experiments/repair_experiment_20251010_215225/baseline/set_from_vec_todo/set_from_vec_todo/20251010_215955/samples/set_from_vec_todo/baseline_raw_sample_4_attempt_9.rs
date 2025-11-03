verus! {

use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == set i | 0 <= i < self.vt@.len() => self.vt@[i]
    {
        set i | 0 <= i < self.vt@.len() => self.vt@[i]
    }

    pub fn new() -> (s: Self)
        requires
            true,
        ensures
            s.view() == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self.view() == old(self).view().union(set![v]),
    {
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self.view().contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt@.len(),
                forall j: nat :: j < i ==> self.vt@[j] != v,
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
