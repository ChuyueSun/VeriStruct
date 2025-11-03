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
        requires
            true,
        ensures
            self@ =~= old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // We show that the new vector is the old vector plus [v]
            assert(self.vt@ =~= old(self).vt@ + seq![v]);
            // Next, prove that the set view is the old set plus v
            assert forall|x: u64|
                self@.contains(x)
                <==> old(self).view().insert(v).contains(x)
            by {
                reveal(self@);
                reveal(old(self).view());
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> v in self@,
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall|j: int| (0 <= j && j < i) ==> (self.vt[j] != v)
        {
            if self.vt[i] == v {
                return true;
            }
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
