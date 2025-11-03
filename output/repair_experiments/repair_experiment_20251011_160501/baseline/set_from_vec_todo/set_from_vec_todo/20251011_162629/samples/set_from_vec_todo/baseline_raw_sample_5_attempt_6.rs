use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == set j: int | 0 <= j < self.vt@.len() => self.vt@[j]
    {
        set j: int | 0 <= j < self.vt@.len() => self.vt@[j]
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self)@ union set![v]
    {
        proof {
            // No additional proof steps needed
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v in self@)
    {
        let mut found = false;
        let len = self.vt.len();
        for i in 0..len
            invariant
                0 <= i <= len,
                found ==> exists j in 0..i { self.vt@[j] == v },
                !found ==> forall j in 0..i { self.vt@[j] != v },
        {
            if self.vt[i] == v {
                found = true;
            }
        }
        found
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
