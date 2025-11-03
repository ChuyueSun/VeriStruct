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
        ensures
            self@ =~= old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Connect updated vector to updated set:
            // For every x, x is in self.view() <==> x is in old(self).view().insert(v).
            assert forall|x: u64| self.view().contains(x) <==> old(self).view().insert(v).contains(x) by {
                reveal_with_fuel(self.view, 1);
                reveal_with_fuel(old(self).view, 1);

                // (→) If x in self, then either x was already in old(self) or x == v
                // because we pushed v into self.vt.
                // (←) If x in old(self) or x == v, then x is in self.
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                (0 <= i && i <= self.vt.len())
                && (forall|j: usize| (0 <= j && j < i) ==> self.vt[j as int] != v)
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
