use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // The view is defined by which elements in vt@ appear in the set
        Set::new(|x: u64| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![],
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
            // Show that pushing v onto self.vt updates the set by inserting v
            assert(self.vt@ =~= old(self).vt@ + seq![v]);
            // Now connect that to the abstract Set view().
            // The new self@ has all elements of old(self)@ plus v.
            assert forall|x: u64| self.view().contains(x) == (old(self).view().contains(x) || x == v) by {
                // By construction: if x was in old(self).view() or x = v, it's now in new self.view().
                // If x is in new self.view(), then either x was in old(self).view() or x = v.
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                // We haven't found v in any index < i, otherwise we would have returned early
                forall|j: int| 0 <= j < i ==> self.vt[j] != v
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

// VEval Score: Compilation Error: False, Verified: 6, Errors: 0, Verus Errors: 0
