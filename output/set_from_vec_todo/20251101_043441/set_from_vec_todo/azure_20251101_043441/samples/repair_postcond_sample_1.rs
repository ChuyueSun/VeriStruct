use vstd::prelude::*;
use vstd::assert_seqs_equal;
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
            s@ =~= set![]
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.insert(v)
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            assert_seqs_equal!(self.vt@, old(self).vt@ + seq![v]);
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v)
    {
        for i in
            0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
        {
            if self.vt[i] == v {
                return true;
            }
        }
        proof {
            assert(!self@.contains(v)); // Added by AI
        }
        false
    }
}

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

// VEval Score: Compilation Error: False, Verified: 5, Errors: 1, Verus Errors: 1
