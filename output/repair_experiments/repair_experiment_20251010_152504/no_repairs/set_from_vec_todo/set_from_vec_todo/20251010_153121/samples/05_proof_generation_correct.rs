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
            // After push, self.vt@ = old(self).vt@ + seq![v], which exactly corresponds
            // to old(self)@.insert(v) in the spec representation.
            // Hence, self@ =~= old(self)@.insert(v).
            assert(self@ =~= old(self)@.insert(v));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> self@.contains(v),
    {
        for i in 0..self.vt.len()
            invariant
                i <= self.vt.len(),
                // If we haven't returned yet, then we haven't found `v` in any position < i
                forall|k: int| 0 <= k < i ==> self.vt[k] != v,
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
