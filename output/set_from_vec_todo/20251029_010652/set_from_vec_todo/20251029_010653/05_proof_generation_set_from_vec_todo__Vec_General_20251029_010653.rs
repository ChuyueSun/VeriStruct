use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= Set::<u64>::empty()
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v)
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // First, establish the new sequence is old(...) plus seq![v].
            assert(self.vt@ == old(self).vt@ + seq![v]);
            // Now show that the resulting set is old(...) plus the element v.
            assert forall|x: u64| self@.contains(x) == old(self)@.insert(v).contains(x) by {
                // x in self@  <==>  self.vt@.contains(x)
                //            <==>  (old(self).vt@ + seq![v]).contains(x)
                //            <==>  old(self).vt@.contains(x) || x == v
                // that is exactly old(self)@.insert(v).contains(x).
            }
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall|j: int| 0 <= j && j < i ==> self.vt[j] != v,
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

// Step 5 (proof_generation) VEval Score: Compilation Error: False, Verified: 6, Errors: 0, Verus Errors: 0
// Verified: 6, Errors: 0, Verus Errors: 0
