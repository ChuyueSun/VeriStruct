use vstd::prelude::*;
use vstd::seq_lib::seq;
use vstd::set::*;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        seq::to_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        requires
            true,
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self@ == old(self)@.union(set![v]),
    {
        proof {
            // Trivial proof that pushing v adds it to the end of vt, updating self@ accordingly
            // No explicit assertions needed here for this simple example
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained ==> v in self@,
            !contained ==> v !in self@,
    {
        for i in 0..self.vt.len()
            invariant
                // i always between 0 and vt.len()
                0 <= i && i <= self.vt.len(),
                // for all j in [0..i), vt[j] != v if we haven't returned true yet
                forall j in 0..i => self.vt[j] != v
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
