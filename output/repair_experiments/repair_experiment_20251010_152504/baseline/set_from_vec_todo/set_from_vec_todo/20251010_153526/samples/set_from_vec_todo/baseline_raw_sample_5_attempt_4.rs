use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures result == self.vt@.to_set()
    {
        self.vt@.to_set()
    }

    pub fn new() -> (s: Self)
        ensures s@ == set![],
        ensures s.vt.len() == 0
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures self@ == old(self)@ union set!{v},
        ensures self.vt.len() == old(self.vt.len()) + 1
    {
        proof {
            // No additional proof steps needed
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained == (v in self@)
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                forall |j| 0 <= j < i ==> self.vt.index_spec(j) != v
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
