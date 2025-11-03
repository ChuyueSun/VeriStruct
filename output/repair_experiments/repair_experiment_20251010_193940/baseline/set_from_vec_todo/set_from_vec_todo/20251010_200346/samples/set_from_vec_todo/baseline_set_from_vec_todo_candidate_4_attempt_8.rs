use vstd::prelude::*;
use vstd::seq_lib::{seq_to_set, group_seq_properties};

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures result == seq_to_set(self.vt@)
    {
        seq_to_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures s.view() == Set::<u64>::empty()
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures self.view() == old(self).view().union(set![v])
    {
        #[verifier::proof] {
            group_seq_properties::lemma_seq_to_set_push(old(self).vt@, v);
            assert(seq_to_set(old(self).vt@ + [v]) == seq_to_set(old(self).vt@).union(set![v]));
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained <==> v ∈ self.view()
    {
        for i in 0..self.vt.len()
            invariant
                self.vt@ == old(self).vt@,
                forall|j: int| 0 <= j < i ==> self.vt@[j] != v
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
