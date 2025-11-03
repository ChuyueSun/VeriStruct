use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // Replaces the TODO
        set i | i in self.vt@.index_range() => self.vt@[i]
    }

    pub fn new() -> (s: Self)
        // Replaces the TODO
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        // Replaces the TODO
        ensures
            self@ == old(self)@.union(set![v]),
    {
        // Replaces the TODO
        proof {
            // Verus automatically checks that pushing v
            // adjusts the abstract set accordingly.
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        // Replaces the TODO
        ensures
            contained == (v in self@),
    {
        let mut i = 0;
        // Replaces the TODO (loop invariant)
        while i < self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                forall j in 0..i { self.vt[j] != v },
        {
            if self.vt[i] == v {
                return true;
            }
            i = i + 1;
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
