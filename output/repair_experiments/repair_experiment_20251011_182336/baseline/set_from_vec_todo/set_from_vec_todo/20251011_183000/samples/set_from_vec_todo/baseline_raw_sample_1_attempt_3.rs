use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::from_seq(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self)@ union set![v],
    {
        self.vt.push(v);
        proof {
            assert(Set::from_seq(old(self.vt)@) union set![v] == Set::from_seq(self.vt@));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v in self@),
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall k in 0..i => self.vt[k] != v,
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
