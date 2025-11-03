use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        { x: u64 | exists i: int :: (0 <= i < self.vt@.len()) && x == self.vt@[i] }
    }

    pub fn new() -> (s: Self)
        // TODO: add requires and ensures
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        // TODO: add requires and ensures
    {
        // TODO: add proof
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        // TODO: add requires and ensures
    {
        for i in iter: 0..self.vt.len()
        // TODO: add invariant
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
