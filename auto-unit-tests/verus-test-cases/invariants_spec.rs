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