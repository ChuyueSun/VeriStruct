use vstd::prelude::*;
use vstd::seq_lib::*;

verus! {

#[verifier::spec]
fn seq_unique_set(seq: Seq<u64>) -> Set<u64> {
    set x | x in seq
}

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64>
        ensures
            result == seq_unique_set(self.vt@),
    {
        seq_unique_set(self.vt@)
    }

    pub fn new() -> (s: Self)
        ensures
            s.view() =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self.view() == old(self).view().union(set![v]),
    {
        #[verifier::proof] {
            assert(self.vt@ == old(self).vt@ + seq![v]);
            assert(seq_unique_set(self.vt@) == seq_unique_set(old(self).vt@ + seq![v]));
            assert(seq_unique_set(old(self).vt@ + seq![v]) == seq_unique_set(old(self).vt@).union(set![v]));
        }
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self.view().contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i <= self.vt.len(),
                forall j in 0..i => self.vt[j] != v,
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
