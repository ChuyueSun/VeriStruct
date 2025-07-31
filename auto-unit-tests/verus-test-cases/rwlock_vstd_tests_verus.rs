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
pub fn main() {
    test_lock_even();
    test_lock_odd();
    test_lock_arbitrary();
}

fn test_lock_even() {
    let lock_even = RwLock::<u64>::new(20);
    let read_handle_even = lock_even.read().unwrap();
    let val_even = *read_handle_even;
    assert(val_even % 2 == 0);
}

fn test_lock_odd() {
    let lock_odd = RwLock::<u64>::new(23);
    let read_handle_odd = lock_odd.read().unwrap();
    let val_odd = *read_handle_odd;
    assert(val_odd % 2 == 1);
}

fn test_lock_arbitrary() {
    let n: u64 = 42;
    let lock_arbitrary = RwLock::<u64>::new(n);
    let read_handle_arbitrary = lock_arbitrary.read().unwrap();
    let val_arbitrary = *read_handle_arbitrary;
    assert(val_arbitrary % 2 == n % 2);
}
} // verus!