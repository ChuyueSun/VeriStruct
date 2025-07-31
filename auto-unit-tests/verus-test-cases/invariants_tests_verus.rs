#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::invariant::*;

verus! {

struct ModPredicate {}

impl InvariantPredicate<int, u32> for ModPredicate {
    closed spec fn inv(k: int, v: u32) -> bool {
        v as int % 2 == k
    }
}

/* TEST CODE BELOW */
pub fn main() {
    // Test even number: 4 % 2 = 0
    assert(ModPredicate::inv(0, 4));
    assert(!ModPredicate::inv(1, 4));

    // Test odd number: 7 % 2 = 1
    assert(ModPredicate::inv(1, 7));
    assert(!ModPredicate::inv(0, 7));
}
} // verus!