use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64|
            exists|i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x
        )
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.union(set![v]),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;

            // We'll prove that the new set of elements in `self`
            // is exactly the old set plus `v`.
            assert forall |x: u64| self@.contains(x) == old(self)@.union(set![v]).contains(x)
            by {
                // Forward direction:
                // If x is in self@, there is some index i with self.vt@[i] == x.
                // If i < old(self).vt@.len(), then x was already in old(self)@.
                // If i == old(self).vt@.len(), then x == v.
                // Reverse direction:
                // If x is in old(self)@.union(set![v]), then x was in old(self)@
                // or x == v, so x must be in the updated self@ by construction.
            }
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall |j: int| 0 <= j && j < i ==> self.vt@[j] != v
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
