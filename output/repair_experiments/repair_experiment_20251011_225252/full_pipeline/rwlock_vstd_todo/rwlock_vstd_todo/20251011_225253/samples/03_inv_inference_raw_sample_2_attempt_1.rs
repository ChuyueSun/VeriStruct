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
        (v as int % 2) == self.parity
    }
}

/* TEST CODE BELOW */

fn test1(n: u64) {
    let lock_even = RwLock::<u64, FixedParity>::new(20, Ghost(FixedParity { parity: 0 }));
    let lock_odd = RwLock::<u64, FixedParity>::new(23, Ghost(FixedParity { parity: 1 }));

    let read_handle_even = lock_even.acquire_read();
    let val_even = *read_handle_even.borrow();
    // assert(val_even % 2 == 0);

    let read_handle_odd = lock_odd.acquire_read();
    let val_odd = *read_handle_odd.borrow();
    // assert(val_odd % 2 == 1);

    let lock_arbitrary = RwLock::<u64, FixedParity>::new(n, Ghost(FixedParity { parity: (n % 2) as int }));
    let read_handle_arbitrary = lock_arbitrary.acquire_read();
    let val_arbitrary = *read_handle_arbitrary.borrow();
    // assert(val_arbitrary % 2 == n % 2);
}

fn test2(n: u64) {
    let lock_even = RwLock::<u64, FixedParity>::new(20, Ghost(FixedParity { parity: 0 }));
    let lock_odd = RwLock::<u64, FixedParity>::new(23, Ghost(FixedParity { parity: 1 }));

    let read_handle_even = lock_even.acquire_read();
    let val_even = *read_handle_even.borrow();
    assert(val_even % 2 == 0);

    let read_handle_odd = lock_odd.acquire_read();
    let val_odd = *read_handle_odd.borrow();
    // assert(val_odd % 2 == 1);

    let lock_arbitrary = RwLock::<u64, FixedParity>::new(n, Ghost(FixedParity { parity: (n % 2) as int }));
    let read_handle_arbitrary = lock_arbitrary.acquire_read();
    let val_arbitrary = *read_handle_arbitrary.borrow();
    // assert(val_arbitrary % 2 == n % 2);
}

fn test3(n: u64) {
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
    // assert(val_arbitrary % 2 == n % 2);
}

fn test4(n: u64) {
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
