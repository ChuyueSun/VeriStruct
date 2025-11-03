use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self@ =~= old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Now self.vt@ == old(self).vt@ + seq![v]
            // Show self@ == old(self)@.insert(v).
            // The membership of x in self@ is precisely the membership of x in old(self).view() plus possibly x = v.
            // Thus we match the definition of old(self)@.insert(v).
            assert(self@ =~= old(self)@.insert(v));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> v in self@,
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall|j: int| (0 <= j && j < i) ==> (self.vt[j] != v)
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

// Repair Round 9 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
