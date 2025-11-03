use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // We've defined the view to rely on vt@.contains(x)
        // so it precisely tracks the set of elements within vt.
        Set::new(|x: u64| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        ensures
            s.view() =~= Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64) -> (ret: ())
        ensures
            self.view() =~= old(self).view().insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Show that the updated sequence is old(...) plus the single element v
            assert(self.vt@ =~= old(self).vt@ + seq![v]);
            // That means that for any x, membership is old(...) or x == v
            assert(forall|x: u64| self.vt@.contains(x) == (old(self).vt@.contains(x) || x == v));
            // Conclude that the new view is old(...) plus v as a set
            assert(self.view() =~= old(self).view().insert(v));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self.view().contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                self.vt.len() == old(self).vt.len(),
                0 <= i <= self.vt.len(),
                self.view() =~= old(self).view(),
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
