use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|v: u64| self.vt@.contains(v))
    }

    pub fn new() -> (s: Self)
        requires
            true,
        ensures
            s.view() =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self.view() =~= old(self).view().union(set![v]),
    {
        #[verifier::proof] {
            let old_seq = self.vt@;
        }
        self.vt.push(v);
        #[verifier::proof] {
            assert(self.vt@ =~= old_seq + [v]);
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self.view().contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                forall |j: int| 0 <= j < i ==> self.vt[j] != v,
            decreases self.vt.len() - i
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

// Checkpoint Best VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
// Compilation Error: True
