use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    //─────────────────────────
    // 1) VIEW FUNCTION
    //─────────────────────────
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            // The resulting set exactly corresponds to the elements in vt@
            forall|x: u64| result.contains(x) <==> self.vt@.contains(x),
    {
        Set::new(|x: u64| self.vt@.contains(x))
    }

    //─────────────────────────
    // 2) CONSTRUCTOR
    //─────────────────────────
    pub fn new() -> (s: Self)
        requires
            // No particular preconditions
        ensures
            s@ =~= set![],
            s.vt.len() == 0,
    {
        VecSet { vt: Vec::new() }
    }

    //─────────────────────────
    // 3) INSERT METHOD
    //─────────────────────────
    pub fn insert(&mut self, v: u64)
        requires
            // No particular preconditions
        ensures
            self@ =~= old(self)@.union(set![v]),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Here, we establish that vt@ is old(self).vt@ + seq![v]
            // which implies self.view() is old(self).view() ∪ set![v].
            assert forall|x: u64| self@.contains(x) <==> old(self)@.union(set![v]).contains(x);
        }
    }

    //─────────────────────────
    // 4) CONTAINS METHOD
    //─────────────────────────
    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            // No particular preconditions
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                // All indices checked so far do not have v;
                // if we exit the loop normally, it means none matched v.
                forall|j: int| 0 <= j < i ==> self.vt[j] != v
        {
            if self.vt[i] == v {
                return true;
            }
        }
        false
    }
}

/* TEST CODE BELOW */

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

// Final VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
