#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::rwlock::*;

verus!{

// Using higher-order functions is often cumbersome, we can use traits instead.

struct FixedParity {
    pub parity: int,
}

impl RwLockPredicate<u64> for FixedParity {
    closed spec fn inv(self, v: u64) -> bool {
        v % 2 == self.parity
    }
}

/* TEST CODE BELOW */