/// Top-level docstring can go here if desired

use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// The logical View of this structure is a set of all elements stored in `vt`.
    /// Because duplicates in the Vec do not affect membership in the set,
    /// we represent the entire collection as:
    ///
    ///   Set::new(|x: u64| exists i: int {
    ///       0 <= i && i < self.vt@.len() && self.vt@[i] == x
    ///   })
    ///
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| exists|i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x )
    }

    /// Creates a new, empty VecSet.
    /// ensures the resulting set is empty
    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![]
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts a value into the set.
    /// ensures the resulting set is the old set plus `v`
    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v)
    {
        self.vt.push(v);

        // Proof that self@ now equals old(self).view().insert(v).
        proof {
            broadcast use group_seq_properties;
            // We show that membership in self@ matches membership in old(self).view().insert(v).
            assert forall|x: u64| self@.contains(x) <==> old(self).view().insert(v).contains(x) by {
                // "self@.contains(x)" expands to:
                //   exists i in [0 .. self.vt@.len()) s.t. self.vt@[i] == x
                // Compare with old(self).view().insert(v).contains(x),
                // which means x is either in old(self).view() or x == v.
                //
                // Because self.vt@.len() == old(self).vt@.len() + 1,
                // - If x was already in old(self), it remains in self.
                // - If x == v, then it appears at the new index (old length).
            };
            assert(self@ =~= old(self).view().insert(v));
        }
    }

    /// Returns true if `v` is in the set, false otherwise.
    /// ensures contained == self@.contains(v)
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                i <= self.vt.len(),
                // No changes to self here, so self@ remains stable.
                // We keep track that we haven't found `v` before index i.
                forall|j: int| 0 <= j && j < i ==> self.vt@[j] != v,
            decreases (self.vt.len() - i)
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

// VEval Score: Compilation Error: False, Verified: 5, Errors: 1, Verus Errors: 1
