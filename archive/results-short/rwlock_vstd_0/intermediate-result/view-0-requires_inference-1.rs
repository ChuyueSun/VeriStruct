#![allow(unused_imports)]

use vstd::prelude::*;
use vstd::rwlock::*;

verus!{

// Using higher-order functions is often cumbersome, we can use traits instead.

struct FixedParity {
    pub parity: int,
}

impl RwLockPredicate<u64> for FixedParity {
    open spec fn inv(self, v: u64) -> ( bool ) as bool {
        // TODO: add specification
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
// VerusErrorType.Other: expected curly braces
// {"$message_type":"diagnostic","message":"expected curly braces","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpmwfky854","byte_start":297,"byte_end":299,"line_start":15,"line_end":15,"column_start":48,"column_end":50,"is_primary":true,"text":[{"text":"    open spec fn inv(self, v: u64) -> ( bool ) as bool {","highlight_start":48,"highlight_end":50}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected curly braces\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpmwfky854:15:48\n   |\n15 |     open spec fn inv(self, v: u64) -> ( bool ) as bool {\n   |                                                ^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 