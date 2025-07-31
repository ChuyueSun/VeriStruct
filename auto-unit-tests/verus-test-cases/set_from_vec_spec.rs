use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        self.vt@.to_set()
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
            broadcast use group_seq_properties;
            assert(self.vt@ =~= old(self).vt@ + seq![v]);
        }
        
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained == self@.contains(v),
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

/* TEST CODE BELOW */