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
            broadcast_use group_seq_properties;
            // Before the push, prove the final set is old(self) plus v:
            // 1) After the push, self.vt@ == old(self).vt@ + seq![v]
            assert(self.vt@ == old(self).vt@ + seq![v]);
            // 2) Extensional set equality: membership in self@ is membership in old(self)@ or x == v
            assert forall|x: u64| #[trigger] self@.contains(x) == old(self)@.insert(v).contains(x) by {
                assert(self@.contains(x) == self.vt@.contains(x));
                assert(old(self).view().insert(v).contains(x) == (old(self).vt@.contains(x) || x == v));
                // So self.vt@.contains(x) <==> old(self).vt@.contains(x) || x == v
            }
        }
        self.vt.push(v);
    }

    /// Checks if the given value is contained in the VecSet.
    /// ensures contained == self@.contains(v)
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in
            0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                // We haven't found v in any index [0..i)
                forall|k: int| 0 <= k && k < i ==> self.vt[k] != v,
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
