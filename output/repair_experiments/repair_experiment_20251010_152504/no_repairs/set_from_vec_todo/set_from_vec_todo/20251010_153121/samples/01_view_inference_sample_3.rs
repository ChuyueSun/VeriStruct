use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// Returns the mathematical set of all u64 values in `vt`.
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| exists|i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x)
    }

    /// Creates a new, empty VecSet.
    /// ensures the returned VecSet's view is empty.
    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts the value `v` into the VecSet.
    /// ensures the resulting view is the old view union {v}.
    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v),
    {
        // push the value into the underlying vector
        self.vt.push(v);
    }

    /// Checks if the value `v` is contained in the VecSet.
    /// ensures `contained` is true exactly when `v` is in the view.
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v in self@),
    {
        for i in iter: 0..self.vt.len()
            invariant
                i <= self.vt.len(),
                // If we have not yet returned, we haven't found `v` in any index < i.
                forall|j: int|
                    0 <= j < i ==>
                    self.vt@[j] != v,
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
