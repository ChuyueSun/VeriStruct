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
            // Show that adding 'v' to the vector changes the set by inserting 'v' into the old set
            assert forall|x: u64| self.view().contains(x)
                <==> old(self).view().contains(x) || x == v
            by {
                // If self.view().contains(x), then self.vt@.contains(x). Since we've
                // added exactly one new element 'v', either x was already contained
                // in old(self).vt@ or x is exactly v.
                // Conversely, if old(self).view().contains(x) or x == v, then x is
                // in self.vt@ after the push.
            };
            assert(self@ =~= old(self)@.insert(v));
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> v in self@,
    {
        for i in iter: 0..self.vt.len()
            invariant (forall|j| (0 <= j && j < i) ==> (self.vt[j] != v))
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

}

// Repair Round 8 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1

} // verus!
