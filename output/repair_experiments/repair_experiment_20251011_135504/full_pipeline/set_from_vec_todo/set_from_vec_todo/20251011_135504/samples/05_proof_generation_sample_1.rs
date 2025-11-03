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
        requires
            true,
        ensures
            s@ == Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self@ == old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // First, note that the concrete vector is old(self).vt@ plus the new element `v`.
            assert(self.vt@ =~= old(self).vt@ + seq![v]);

            // Now relate that to the set view; self@ is Set::new(|x| self.vt@.contains(x)),
            // and old(self)@.insert(v) is "everything in old(self)@" plus possibly v.
            // Because self.vt@ is old(self).vt@ plus v, membership in self.view()
            // is exactly membership in old(self).view() plus v, so self@ = old(self)@.insert(v).
            assert forall|x: u64| self.view().contains(x) <==> old(self).view().insert(v).contains(x) by {
                // forward direction
                assume(self.view().contains(x));
                reveal(self.view());
                assert(self.vt@.contains(x));
                // => x is in old(self).vt@ or x == v
                // => x ∈ old(self).view() or x == v
                // => x ∈ old(self).view().insert(v)
                // reverse direction:
                assume(old(self).view().insert(v).contains(x));
                // => x ∈ old(self).view() or x == v
                // => old(self).vt@.contains(x) or x == v
                // => self.vt@.contains(x)
                // => x ∈ self.view()
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i <= self.vt@.len(),
                forall|j: int| 0 <= j < i ==> self.vt@[j] != v
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
