use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// Returns the abstract set of elements stored in this VecSet.
    pub closed spec fn view(&self) -> Set<u64> {
        Set::empty().union_over(self.vt@.index_range(), |i| self.vt@[i])
    }

    /// Creates a new, empty VecSet.
    pub fn new() -> (s: Self)
        requires
            true,
        ensures
            s.view() == Set::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts v into the VecSet.
    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self.view() == old(self).view().insert(v),
    {
        proof {
            // After push, vt@ becomes old(self).vt@ + [v].
            // Therefore, self.view() becomes old(self).view().insert(v).
        }
        self.vt.push(v);
    }

    /// Returns true if v is contained in the VecSet, false otherwise.
    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self.view().contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                // If we haven't returned yet, it must be that none of the previous elements matched v.
                forall j in 0..i => self.vt[j] != v
        {
            if self.vt[i] == v {
                return true;
            }
        }
        proof {
            // Because we didn't return, we never found v in self.vt.
            // Thus, v is not in self.view().
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
