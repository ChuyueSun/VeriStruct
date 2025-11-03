#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::rwlock::*;

verus!{

struct FixedParity {
    pub parity: int,
}

impl RwLockPredicate<u64> for FixedParity {
    closed spec fn inv(self, v: u64) -> bool {
        v % 2 == self.parity
    }
}

/* TEST CODE BELOW */

fn test(n: u64) {
    let lock_even = RwLock::<u64, FixedParity>::new(20, Ghost(FixedParity { parity: 0 }));
    let lock_odd = RwLock::<u64, FixedParity>::new(23, Ghost(FixedParity { parity: 1 }));

    let read_handle_even = lock_even.acquire_read();
    let val_even = *read_handle_even.borrow();
    assert(val_even % 2 == 0);

    let read_handle_odd = lock_odd.acquire_read();
    let val_odd = *read_handle_odd.borrow();
    assert(val_odd % 2 == 1);

    let lock_arbitrary = RwLock::<u64, FixedParity>::new(n, Ghost(FixedParity { parity: (n % 2) as int }));
    let read_handle_arbitrary = lock_arbitrary.acquire_read();
    let val_arbitrary = *read_handle_arbitrary.borrow();
    assert(val_arbitrary % 2 == n % 2);
}

pub fn main() {
}

}

// BASELINE BEST CODE
// Source: Candidate 1, Attempt 1
// Score: Verified=2, Errors=0, Verus Errors=0
// Compilation Error: False
// Is Correct: True
// Timestamp: 2025-10-10T21:59:11.986288
