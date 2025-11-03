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
        requires
            true,
        ensures
            s@ == Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        requires
            true,
        ensures
            self@ == old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Connect the new self.vt@ to the old one plus seq![v]
            assert(self.vt@ == old(self).vt@ + seq![v]);

            // Show that the resulting set of elements matches old(self).view().insert(v)
            assert forall|x: u64| #[trigger] self.view().contains(x) == old(self).view().insert(v).contains(x) by {
                assert(self.view().contains(x) == self.vt@.contains(x));
                assert(old(self).view().insert(v).contains(x) == (old(self).vt@.contains(x) || x == v));
                assert(self.vt@.contains(x) == (old(self).vt@ + seq![v]).contains(x));
            }
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        requires
            true,
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i <= self.vt@.len(),
                forall|j: int| 0 <= j < i ==> self.vt@[j] != v
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


/* TEST CODE BELOW */

fn test1(t: Vec<u64>)
{
    let mut vs: VecSet = VecSet::new();
    // assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    // assert(!contains2);
    let contains3 = vs.contains(3);
    // assert(contains3);
    // assert(vs@ =~= set![3, 5]);
}

fn test2(t: Vec<u64>)
{
    let mut vs: VecSet = VecSet::new();
    assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    // assert(!contains2);
    let contains3 = vs.contains(3);
    // assert(contains3);
    // assert(vs@ =~= set![3, 5]);
}

fn test3(t: Vec<u64>)
{
    let mut vs: VecSet = VecSet::new();
    assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    assert(!contains2);
    let contains3 = vs.contains(3);
    // assert(contains3);
    // assert(vs@ =~= set![3, 5]);
}

fn test4(t: Vec<u64>)
{
    let mut vs: VecSet = VecSet::new();
    assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    assert(!contains2);
    let contains3 = vs.contains(3);
    assert(contains3);
    // assert(vs@ =~= set![3, 5]);
}

fn test5(t: Vec<u64>)
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

// Checkpoint Best VEval Score: Compilation Error: False, Verified: 6, Errors: 0, Verus Errors: 0
// Verified: 6, Errors: 0, Verus Errors: 0
// Compilation Error: False
