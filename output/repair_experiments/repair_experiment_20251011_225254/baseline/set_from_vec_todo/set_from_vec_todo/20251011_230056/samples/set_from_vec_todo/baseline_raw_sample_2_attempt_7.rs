use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        set i in 0..self.vt.len() | self.vt[i]
    }

    pub fn new() -> (s: Self)
        ensures s.view() =~= set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self.view() =~= old(self).view().union(set![v])
    {
        proof {
            assert(self.view().union(set![v]) =~= old(self).view().union(set![v]));
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self.view().contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                forall j: nat :: j < i ==> self.vt[j] != v,
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
