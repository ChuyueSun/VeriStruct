use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            forall |x: u64| x ∈ result <==> exists |i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x,
    {
        Set::new(|x: u64| exists |i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x)
    }

    pub fn new() -> (s: Self)
        ensures
            s.view() == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self.view() == old(self).view().union(set![v]),
    {
        proof {
            assert(old(self).view() == Set::new(|x: u64| exists |i: int| 0 <= i < old(self).vt@.len() && old(self).vt@[i] == x));
        }
        self.vt.push(v);
        proof {
            assert(self.view() == Set::new(|x: u64| exists |i: int| 0 <= i < self.vt@.len() && self.vt@[i] == x));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v ∈ self.view()),
    {
        let mut i = 0;
        while i < self.vt.len()
            invariant
                0 <= i <= self.vt@.len(),
                forall |j: int| 0 <= j < i ==> self.vt@[j] != v,
        {
            if self.vt[i] == v {
                return true;
            }
            i += 1;
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
