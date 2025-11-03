use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// The mathematical set view of the elements in `vt`.
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == Set::new(|x: u64| self.vt@.contains(x)),
    {
        Set::new(|x: u64| self.vt@.contains(x))
    }

    /// Creates a new, empty `VecSet`.
    pub fn new() -> (s: Self)
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts `v` into the set.
    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self@).union(set![v]),
    {
        #[verifier::proof] let before_insert = self.view();
        self.vt.push(v);
        #[verifier::proof] {
            assert(self.view() == before_insert.union(set![v]));
        }
    }

    /// Returns true if `v` is contained in the set.
    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == (v in self@),
    {
        let mut result = false;
        for i in 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                // No match found so far:
                forall |j: int| 0 <= j < i ==> self.vt[j] != v,
        {
            if self.vt[i] == v {
                result = true;
                return result;
            }
        }
        result
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
