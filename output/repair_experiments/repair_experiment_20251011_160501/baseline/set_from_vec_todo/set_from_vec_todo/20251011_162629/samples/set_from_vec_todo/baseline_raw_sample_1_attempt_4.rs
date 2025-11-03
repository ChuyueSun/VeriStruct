use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        ensures(|result: Set<u64>|
            forall|x: u64| x ∈ result <==> x in self.vt@
        );
        Set::new(self.vt@.map(|x: u64| x))
    }

    pub fn new() -> (s: Self)
        ensures s@ === set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures self@ == old(self@) ∪ set![v]
    {
        let old_view = old(self@);
        self.vt.push(v);
        proof {
            let old_seq = old(self.vt@);
            let new_seq = self.vt@;
            group_seq_properties::<u64>(old_seq + seq![v]);
            assert(new_seq == old_seq + seq![v]);
            assert(self@ == old_view ∪ set![v]);
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures contained == (v ∈ self@)
    {
        for i in (0..self.vt.len())
            invariant
                0 <= i <= self.vt.len(),
                forall|j: usize| j < i ==> self.vt[j] != v
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
