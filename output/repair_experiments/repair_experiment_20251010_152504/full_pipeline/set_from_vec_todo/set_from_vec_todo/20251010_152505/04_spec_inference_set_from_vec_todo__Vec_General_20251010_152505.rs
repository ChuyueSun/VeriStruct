use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // Spec functions cannot have requires/ensures
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
        // TODO: add proof
        self.vt.push(v);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
        // TODO: add invariant
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

// Step 4 (spec_inference) VEval Score: Compilation Error: False, Verified: 4, Errors: 2, Verus Errors: 2
// Verified: 4, Errors: 2, Verus Errors: 2
