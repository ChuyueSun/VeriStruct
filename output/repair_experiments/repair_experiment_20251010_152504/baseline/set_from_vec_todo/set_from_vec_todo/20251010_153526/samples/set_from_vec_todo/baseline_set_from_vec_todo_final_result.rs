use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;
use vstd::seq_lib::to_set;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == to_set(self.vt@)
    {
        to_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s.view() == Set::<u64>::empty()
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self.view() == old(self).view().union(set![v])
    {
        let old_view = self.view();
        self.vt.push(v);
        proof {
            assert(self.view() == to_set(self.vt@));
            assert(self.view() == old_view.union(set![v]));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self.view().contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                forall |j: int|
                    0 <= j < i ==> self.vt@.index(j) != v
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

// BASELINE FINAL RESULT - set_from_vec_todo
// Total Attempts: 10
// Total Candidates: 1
// Final Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Success: False
// Completed: 2025-10-10T15:45:28.249753
