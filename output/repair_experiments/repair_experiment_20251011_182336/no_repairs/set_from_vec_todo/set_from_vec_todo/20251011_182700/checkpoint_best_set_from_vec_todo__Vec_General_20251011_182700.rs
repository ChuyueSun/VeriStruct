use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x| self.vt@.contains(x))
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self)@.insert(v),
    {
        self.vt.push(v);
        proof {
            broadcast use group_seq_properties;
            // Show that the new state equals the old state plus v
            // i.e., for any x, x in self@ <==> x in old(self)@ or x == v
            assert forall|x: u64| self.view().contains(x)
                == (old(self).view().contains(x) || x == v)
            by {
                // The newly pushed element is exactly v, so
                // any other x comes from the old set membership
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                i <= self.vt@.len(),
                forall|j: int| 0 <= j < i ==> self.vt@[j] != v,
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
