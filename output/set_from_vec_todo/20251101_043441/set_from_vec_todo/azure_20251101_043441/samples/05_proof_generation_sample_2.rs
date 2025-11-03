/// A simple set of u64 values stored in a Vec<u64>.
/// Demonstrates a View returning a Set<u64>.
use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// Returns the mathematical set of all elements in vt.
    pub closed spec fn view(&self) -> Set<u64> {
        // Each element x is in this set iff x appears in self.vt@
        Set::new(|x: u64| self.vt@.contains(x))
    }

    /// Creates a new, empty VecSet.
    /// ensures s@ =~= set![]
    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![]
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts a value into the VecSet.
    /// ensures self@ =~= old(self)@.insert(v)
    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v)
    {
        proof {
            // No state change has happened yet; nothing to prove here.
            // The real membership argument happens after the push.
        }
        self.vt.push(v);
        proof {
            // Now that `v` is pushed, prove self@ =~= old(self)@.insert(v).
            reveal(self@);
            reveal(old(self).view());
            assert forall|x: u64|
                #[trigger] self@.contains(x)
                <==> old(self).view().insert(v).contains(x)
            by {
                // Case analysis on whether x == v or not:
                // - If x == v, then self.vt@ contains x after push, and old(self).view().insert(v) contains v by definition.
                // - If x != v, then self.vt@ contains x iff old(self).vt@ contained x, which corresponds
                //   to old(self).view().contains(x). So membership is preserved on all other elements.
            };
        }
    }

    /// Checks if the given value is contained in the VecSet.
    /// ensures contained == self@.contains(v)
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in
            // Loop over indices of self.vt
            // invariant: 0 <= i && i <= self.vt.len()
            // This ensures i stays in range.
            0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
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
