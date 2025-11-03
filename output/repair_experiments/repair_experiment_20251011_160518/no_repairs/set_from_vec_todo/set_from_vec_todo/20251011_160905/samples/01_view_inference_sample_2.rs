use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// The mathematical abstraction of this VecSet is a set of the elements contained in `vt`.
    pub closed spec fn view(&self) -> Set<u64> {
        // Minimal representation: all distinct elements in the vector form a set.
        Set::new(|x: u64| self.vt@.contains(x))
    }

    /// Creates a new, empty VecSet.
    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts an element `v`. (Duplicates in the vector do not change the set abstraction.)
    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.union(set![v]),
    {
        // No requires is needed because we allow duplicates in the vector.
        // The set abstraction automatically ignores duplicates.
        // TODO: add proof if needed.
        self.vt.push(v);
    }

    /// Checks if `v` is contained in the set.
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> v ∈ self@,
    {
        for i in 0..self.vt.len()
            invariant
                i <= self.vt@.len(),
                // We haven't found `v` yet if we're still in the loop:
                forall|j: int| 0 <= j < i ==> self.vt@[j] != v,
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
