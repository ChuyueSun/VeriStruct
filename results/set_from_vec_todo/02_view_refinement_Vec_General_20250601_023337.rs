use vstd::prelude::*;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        // Minimal mathematical abstraction of the VecSet data structure
        let v = self.vt@;
        set i in 0..v.len() | v[i]
    }

    pub fn new() -> (s: Self)
        ensures
            s@ == set![],
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ == old(self)@.union(set![v]),
    {
        self.vt.push(v);
        proof {
            broadcast use vstd::seq_lib::group_seq_properties;
        }
        assert(self.vt@ =~= old(self).vt@ + seq![v]);
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> v in self@,
    {
        for i in iter: 0..self.vt.len()
            invariant
                forall|j: nat| j < i ==> self.vt[j as int] != v,
        {
            if self.vt[i] == v {
                return true;
            }
        }
        false
    }
}

fn main() {
    let mut vs: VecSet = VecSet::new();
    assert(vs@ =~= set![]);
    vs.insert(3);
    vs.insert(5);
    let contains2 = vs.contains(2);
    assert(!contains2);
    assert(vs@ =~= set![3, 5]);
}

} // verus!

// Step 2 (view_refinement) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1