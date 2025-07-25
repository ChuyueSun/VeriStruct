use vstd::prelude::*;
use vstd::seq::*;
use vstd::vec::*;

verus! {

struct VecSet {
    vt: Vec<u64>,
}

impl VecSet {
    /// Creates an empty VecSet.
    pub fn new() -> Self
        ensures
            result.vt@.len() == 0,
    {
        VecSet { vt: Vec::new() }
    }

    /// Inserts element `v` into the set.
    /// Duplicates are allowed.
    pub fn insert(&mut self, v: u64)
        ensures
            self.contains(v),
            // The new vector is equal to the old one with v appended.
            self.vt@.len() == old(self.vt@.len()) + 1,
    {
        self.vt.push(v);
    }

    /// Returns true if `v` is contained in the set.
    pub fn contains(&self, v: u64) -> bool
        ensures
            result <==> (exists |i: nat| i < self.vt@.len() && self.vt@.index(i) == v),
    {
        let mut i: usize = 0;
        while i < self.vt.len()
            invariant
                i <= self.vt.len(),
                forall|j: nat| (j < i) ==> (self.vt@.index(j) != v),
            decreases self.vt.len() - i
        {
            if self.vt.index(i) == v {
                return true;
            }
            i = i + 1;
        }
        false
    }
}

/* TEST CODE BELOW */

pub fn main() {
    {
        let set = VecSet::new();
        // Since no elements are inserted, any number should not be contained
        assert(!set.contains(1));
        assert(!set.contains(0));
        assert(!set.contains(u64::MAX));
    }

    {
        let mut set = VecSet::new();
        set.insert(5);
        assert(set.contains(5));
        assert(!set.contains(10));
    }

    {
        let mut set = VecSet::new();
        set.insert(10);
        set.insert(10);
        // Even though the number was inserted twice, contains should return true
        assert(set.contains(10));
    }

    {
        let mut set = VecSet::new();
        let numbers = [1, 2, 3, 4, 5];
        for num in numbers {
            set.insert(num);
        }
        for num in numbers {
            assert(set.contains(num));
        }
        // Check for an element that was not inserted
        assert(!set.contains(6));
    }

    {
        let mut set = VecSet::new();
        // Insert boundary values
        set.insert(0);
        set.insert(u64::MAX);
        assert(set.contains(0));
        assert(set.contains(u64::MAX));
        // Check elements around the edge values to ensure they are not erroneously contained
        assert(!set.contains(1));
        if u64::MAX > 0 {
            assert(!set.contains(u64::MAX - 1));
        }
    }
}

}