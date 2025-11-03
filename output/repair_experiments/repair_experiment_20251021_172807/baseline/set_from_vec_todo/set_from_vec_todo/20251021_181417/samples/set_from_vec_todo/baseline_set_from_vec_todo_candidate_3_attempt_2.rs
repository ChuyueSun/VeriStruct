use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // We define the set of all elements in the vector's ghost view
        set x: u64 | exists i: int, 0 <= i < self.vt@.len() && self.vt@[i] == x
    }

    pub fn new() -> (s: Self)
        ensures
            s.view() == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self.view() == old(self).view().insert(v),
    {
        proof {
            // Push extends the sequence by v, so the resulting set is the old set ∪ {v}.
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self.view().contains(v),
    {
        for i in 0..self.vt.len()
        invariant
            0 <= i <= self.vt.len(),
            forall |k: int|
                0 <= k < i ==> self.vt@.index_spec(k) != v
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
