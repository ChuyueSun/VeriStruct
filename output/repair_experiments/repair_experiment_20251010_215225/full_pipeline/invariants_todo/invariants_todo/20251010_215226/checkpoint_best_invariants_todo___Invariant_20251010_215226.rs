#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::invariant::*;

verus! {

struct ModPredicate {}

impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        (k == 0 ==> v % 2 == 0)
        && (k == 1 ==> v % 2 == 1)
    }
}

/* TEST CODE BEGINS */

pub fn test1() {
    let tracked u: u32 = 5u32;
    let tracked i: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(1, u, 0);
    open_atomic_invariant!(&i => inner => {
        proof {
            if inner == 1u32 {
                inner = 3u32;
            }
            // assert(inner % 2 == 1);
        }
    });
    let tracked j: AtomicInvariant<int, u32, ModPredicate> = AtomicInvariant::new(0, 6u32, 1);
    open_atomic_invariant!(&i => inner_i => {
        open_atomic_invariant!(&j => inner_j => {
            proof {
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
                // assert(inner_i % 2 == 1);
                // assert(inner_j % 2 == 0);
            }
        });
        // assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    // assert(j % 2 == 1);
}

pub fn test2() {
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
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
                // assert(inner_i % 2 == 1);
                // assert(inner_j % 2 == 0);
            }
        });
        // assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    // assert(j % 2 == 1);
}

pub fn test3() {
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
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
                assert(inner_i % 2 == 1);
                // assert(inner_j % 2 == 0);
            }
        });
        // assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    // assert(j % 2 == 1);
}

pub fn test4() {
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
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
                assert(inner_i % 2 == 1);
                assert(inner_j % 2 == 0);
            }
        });
        // assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    // assert(j % 2 == 1);
}

pub fn test5() {
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
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
                assert(inner_i % 2 == 1);
                assert(inner_j % 2 == 0);
            }
        });
        assert(inner_i % 2 == 1);
    });
    let tracked j = i.into_inner();
    // assert(j % 2 == 1);
}

pub fn test6() {
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
                // Change values but maintain invariants:
                // i has k=1 (needs odd), j has k=0 (needs even)
                inner_i = 7u32;  // odd value for k=1
                inner_j = 8u32;  // even value for k=0
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


// Checkpoint Best VEval Score: Compilation Error: False, Verified: 2, Errors: 0, Verus Errors: 0
// Verified: 2, Errors: 0, Verus Errors: 0
// Compilation Error: False
