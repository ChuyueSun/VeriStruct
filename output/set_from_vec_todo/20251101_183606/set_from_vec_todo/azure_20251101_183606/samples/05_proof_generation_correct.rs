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
            broadcast use group_seq_properties;
            // By definition: self@ = Set::new(|x| self.vt@.contains(x)).
            // After push, vt@ = old(self).vt@ + seq![v].
            // So any x is in new self@  <==>  x is in old(self).vt@ or x == v
            //                              <==>  x is in old(self)@.insert(v).
        }
        self.vt.push(v);
    }

    /// Checks if the given value is contained in the VecSet.
    /// ensures contained == self@.contains(v)
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                // We haven't returned yet, so we haven't found `v`
                forall|j: int| 0 <= j && j < i ==> self.vt[j] != v,
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
