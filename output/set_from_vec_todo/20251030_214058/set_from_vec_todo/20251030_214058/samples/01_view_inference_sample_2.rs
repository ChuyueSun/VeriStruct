use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    // View: Logical representation as a set containing exactly
    // those elements that appear in vt.
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| exists|i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x)
    }

    // Creates an empty VecSet.
    pub fn new() -> (s: Self)
        ensures s@ == set![]
    {
        VecSet { vt: Vec::new() }
    }

    // Inserts a value into the set.
    pub fn insert(&mut self, v: u64)
        ensures self@ == old(self)@.union(set![v])
    {
        // Proof: Inserting v (even if already present) yields a set
        // equal to the union with the singleton set containing v.
        self.vt.push(v);
    }

    // Checks if the set contains a given value.
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained == self@.contains(v)
    {
        for i in iter: 0..self.vt.len()
            invariant i <= self.vt.len()
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
