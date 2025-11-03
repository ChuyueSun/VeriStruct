#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::invariant::*;

verus! {

struct ModPredicate {}

impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        if k % 2 == 0 {
            v % 2 == 0
        } else {
            v % 2 == 1
        }
    }
}

/* TEST CODE BEGINS */

pub fn test() {
    let tracked u: u32 = 5u32;
    let tracked i: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(1, u, 0);
    open_atomic_invariant!(&i => inner => {
        proof {
            if inner == 1u32 {
                inner = 3u32;
            }
            assert(inner % 2 == 1);
        }
    });
    let tracked j: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(0, 6u32, 1);
    open_atomic_invariant!(&i => inner_i => {
        open_atomic_invariant!(&j => inner_j => {
            proof {
                inner_i = 7u32;
                inner_j = 8u32;
                assert(inner_i % 2 == 1);
                assert(inner_j % 2 == 0);
            }
        });
        assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    assert(j % 2 == 1);
}
pub fn main() {
}

} // verus!

// BASELINE FINAL RESULT - invariants_todo
// Total Attempts: 1
// Total Candidates: 1
// Final Score: Compilation Error: False, Verified: 2, Errors: 0, Verus Errors: 0
// Success: True
// Completed: 2025-10-11T22:57:06.039096
