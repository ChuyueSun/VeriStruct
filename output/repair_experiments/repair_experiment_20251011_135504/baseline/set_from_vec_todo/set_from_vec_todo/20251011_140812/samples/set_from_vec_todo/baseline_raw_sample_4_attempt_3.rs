use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == set_of_seq(self.vt@)
    {
        set_of_seq(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self@).insert(v)
    {
        proof {
            let old_set = self@;
        }
        self.vt.push(v);
        proof {
            assert(self@ == old_set.insert(v));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v in self@)
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                (forall j: int :: 0 <= j < i ==> self.vt[j] != v)
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
