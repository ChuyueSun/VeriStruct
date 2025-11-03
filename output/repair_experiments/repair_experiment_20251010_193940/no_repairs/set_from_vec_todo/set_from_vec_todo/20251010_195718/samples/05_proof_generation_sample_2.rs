use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    // Added ensures clause describing the exact relationship between the return Set and vt@
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            // For every x, x is in the result iff vt@ contains x
            forall|x: u64| result.contains(x) <==> self.vt@.contains(x),
    {
        Set::new(|x: u64| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        requires
            // no extra preconditions
        ensures
            s@ =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            // no extra preconditions
        ensures
            self@ =~= old(self)@.union(set![v]),
    {
        self.vt.push(v);
        proof {
            // Bridge from the old vector spec to the new one plus v
            broadcast use group_seq_properties;
            assert(self.vt@ =~= old(self).vt@ + seq![v]);

            // Now connect that vector property to the set view
            assert(forall|x: u64| self.view().contains(x)
                <==> (old(self).view().contains(x) || x == v));
            assert(self@ =~= old(self)@.union(set![v]));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            // no extra preconditions
        ensures
            contained == self@.contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                // For all indices j < i, we haven't seen v yet (so vt[j] != v)
                forall|j: int| 0 <= j < i ==> self.vt[j] != v,
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
