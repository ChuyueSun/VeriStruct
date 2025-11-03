use vstd::prelude::*;
use vstd::seq_lib::group_seq_properties;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    pub closed spec fn view(&self) -> Set<u64> {
        Set::new(|x: u64|
            exists|i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x
        )
    }

    pub fn new() -> (s: Self)
        ensures
            s@ =~= Set::<u64>::empty(),
    {
        VecSet { vt: Vec::new() }
    }

    pub fn insert(&mut self, v: u64)
        ensures
            self@ =~= old(self)@.union(set![v]),
    {
        self.vt.push(v);
        proof {
            broadcast_use!(group_seq_properties);
            assert forall|x: u64| (self@.contains(x)) <==> (old(self)@.union(set![v])).contains(x) by {
                // => direction
                if self@.contains(x) {
                    let witness = choose |i: int| 0 <= i && i < self.vt@.len() && self.vt@[i] == x;
                    if witness < old(self).vt@.len() {
                        reveal(old(self));
                        assert(old(self)@.contains(x));
                    } else {
                        assert(x == v);
                    }
                }
                // <= direction
                else {
                    if old(self)@.union(set![v]).contains(x) {
                        if old(self)@.contains(x) {
                            let iw = choose |i: int| 0 <= i && i < old(self).vt@.len() && old(self).vt@[i] == x;
                            assert(0 <= iw && iw < self.vt@.len());
                            assert(self.vt@[iw] == x);
                        } else {
                            assert(x == v);
                            let iw = old(self).vt@.len();
                            assert(0 <= iw && iw < self.vt@.len());
                            assert(self.vt@[iw] == x);
                        }
                    }
                }
            };
        }
    }

    pub fn contains(&self, v: u64) -> (contained: bool)
        ensures
            contained <==> self@.contains(v),
    {
        for i in iter: 0..self.vt.len()
            invariant
                0 <= i && i <= self.vt.len(),
                forall |j: int| 0 <= j && j < i ==> self.vt@[j] != v
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

}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
