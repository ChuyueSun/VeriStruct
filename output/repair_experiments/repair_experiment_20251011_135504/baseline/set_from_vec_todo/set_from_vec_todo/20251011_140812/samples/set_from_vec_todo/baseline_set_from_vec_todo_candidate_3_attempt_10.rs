use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

// Helper function to convert a sequence to a set.
closed spec fn seq_to_set(seq: Seq<u64>) -> Set<u64> {
    decreases(seq.len());
    if seq.len() == 0 {
        Set::empty()
    } else {
        seq_to_set(seq.subrange(0, seq.len() - 1)).insert(seq[seq.len() - 1])
    }
}

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        seq_to_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= Set::empty()
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self@).insert(v)
    {
        proof {
            // Proof obligations are automatically handled by Verus here.
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                old(self@) == self@,
                forall |j: nat| j < i ==> self.vt[j] != v
            decreases
                self.vt.len() - i
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
