#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::rwlock::*;

verus!{

// Using higher-order functions is often cumbersome, we can use traits instead.

struct FixedParity {
    pub parity: int,
}

impl FixedParity {
    open spec fn View(self) -> (bool,) {
        (self.parity == 0,)
    }
}

impl RwLockPredicate<u64> for FixedParity {
    open spec fn inv(self, v: u64) -> bool {
        if self.parity == 0 {
            v % 2 == 0
        } else {
            v % 2 == 1
        }
    }
}

fn example2() {
    let lock_even = RwLock::<u64, FixedParity>::new(20, Ghost(FixedParity { parity: 0 }));
    let lock_odd = RwLock::<u64, FixedParity>::new(23, Ghost(FixedParity { parity: 1 }));

    let read_handle_even = lock_even.acquire_read();
    let val_even = *read_handle_even.borrow();
    assert(val_even % 2 == 0);

    let read_handle_odd = lock_odd.acquire_read();
    let val_odd = *read_handle_odd.borrow();
    assert(val_odd % 2 == 1);
}

pub fn main() {
    example2();
}

}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: function is marked `open` but not marked `pub`; for the body of a function to be visible, the function symbol must also be visible
// {"$message_type":"diagnostic","message":"function is marked `open` but not marked `pub`; for the body of a function to be visible, the function symbol must also be visible","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjqziwg5","byte_start":239,"byte_end":263,"line_start":15,"line_end":15,"column_start":15,"column_end":39,"is_primary":true,"text":[{"text":"    open spec fn View(self) -> (bool,) {","highlight_start":15,"highlight_end":39}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: function is marked `open` but not marked `pub`; for the body of a function to be visible, the function symbol must also be visible\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjqziwg5:15:15\n   |\n15 |     open spec fn View(self) -> (bool,) {\n   |               ^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
