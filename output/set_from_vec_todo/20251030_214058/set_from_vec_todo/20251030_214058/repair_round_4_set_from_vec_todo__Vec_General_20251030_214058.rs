use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// The view of VecSet is the set of its elements.
    pub closed spec fn view(&self) -> Set<u64> {
        set x: u64 | exists i: int . (0 <= i < self.vt@.len() && x == self.vt@[i])
    }

    pub fn new() -> (s: Self)
        ensures s@ == set!{}
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures self@ == old(self)@.insert(v)
    {
        // proof: the new view is the old view with v added.
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained <==> self@.contains(v)
    {
        let mut i: usize = 0;
        while i < self.vt.len()
            invariant(
                i <= self.vt.len() &&
                (forall |j: int| 0 <= j < (i as int) ==> self.vt@[j] != v)
            )
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
    assert(vs@ =~= set!{});
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    assert(!contains2);
    let contains3 = vs.contains(3);
    assert(contains3);
    assert(vs@ =~= set!{3, 5});
}

pub fn main() {}

} // verus!

// Repair Round 4 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
