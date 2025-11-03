use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| self.vt@.contains(x)) // Replaces TODO
    }

    pub fn new() -> (s: Self)
        requires
            true, // Replaces TODO
        ensures
            s@ =~= set![], // Replaces TODO
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true, // Replaces TODO
        ensures
            self@ == old(self)@.union(set![v]), // Replaces TODO
    {
        self.vt.push(v);
        #[verifier::proof_block]
        {
            let old_seq = old(self).vt@;
            let new_seq = self.vt@;
            assert(new_seq.ext_equal(old_seq.push(v)));
            assert(self.view().ext_equal(old(self).view().union(set![v])));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true, // Replaces TODO
        ensures
            contained == self@.contains(v), // Replaces TODO
    {
        for i in (0..self.vt.len())
            invariant
                0 <= i && i <= self.vt.len(), // Replaces TODO
                forall|j: nat| j < i ==> self.vt@.index(j) != v // Replaces TODO
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
