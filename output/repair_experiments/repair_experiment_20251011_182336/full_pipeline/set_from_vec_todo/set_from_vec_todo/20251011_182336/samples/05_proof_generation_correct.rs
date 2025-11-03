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
            // Use all relevant sequence properties
            broadcast use group_seq_properties;

            // Connect the new self@ to old(self)@.insert(v)
            // self@ = Set::new(|x: u64| self.vt@.contains(x))
            // old(self)@ = Set::new(|x: u64| old(self).vt@.contains(x))
            // push(v) means self.vt@ = old(self).vt@ + seq![v]
            assert(self.vt@ =~= old(self).vt@ + seq![v]);
            // Therefore, the set representation is the old set plus 'v'
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                // No changes to self in this &self function, so the length remains constant
                forall|j: usize|
                    0 <= j && j < i ==> self.vt[( j ) as int] != v
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
