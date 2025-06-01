#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use vstd::atomic_ghost::*;
use vstd::prelude::*;
use vstd::{pervasive::*, *};

verus! {

struct_with_invariants!{
    struct Lock<T> {
        field: AtomicBool<_, Option<T>, _>,
    }

    spec fn well_formed(&self) -> bool {
        // The lock is well-formed if and only if
        // the atomic boolean's value corresponds to whether
        // the ghost Option is Some or None.
        (self.field@.value ==> self.field@.ghost.is_Some())
        && (!self.field@.value ==> self.field@.ghost.is_None())
    }
}

impl<T> Lock<T> {
    closed spec fn inv(&self) -> bool {
        (self.field@.value ==> self.field@.ghost.is_Some())
        && (!self.field@.value ==> self.field@.ghost.is_None())
    }
}

// ----------------- REFINED VIEW IMPLEMENTATION ------------------

pub closed trait FlattenLockTupleMethods<T> {
    spec fn is_Some(&self) -> bool;
    spec fn is_None(&self) -> bool;
    spec fn get_Some_0(&self) -> T;
}

impl<T> View for Lock<T> {
    type V = (bool, Option<T>);

    closed spec fn view(&self) -> Self::V {
        (
            self.field@.value,
            if self.field@.value {
                self.field@.ghost
            } else {
                Option::None
            }
        )
    }
}

impl<T> FlattenLockTupleMethods<T> for (bool, Option<T>) {
    closed spec fn is_Some(&self) -> bool {
        self.1.is_Some()
    }

    closed spec fn is_None(&self) -> bool {
        self.1.is_None()
    }

    closed spec fn get_Some_0(&self) -> T {
        self.1.get_Some_0()
    }
}

// -------------------------------------------------------------

fn take<T>(lock: &Lock<T>) -> (t: Tracked<T>)
    requires
        lock.well_formed(),
        old(lock)@.is_Some(),
    ensures
        lock@.is_None(),
        t@ == old(lock)@.get_Some_0(),
{
    loop
        invariant
            lock.well_formed(),
    {
        let tracked ghost_value: Option<T>;
        let result =
            atomic_with_ghost!(
            &lock.field => compare_exchange(true, false);
            update prev -> next;
            ghost g => {
                if prev == true {
                    ghost_value = g;
                    g = Option::None;
                } else {
                    ghost_value = Option::None;
                }
            }
        );
        if let Result::Ok(_) = result {
            return Tracked(
                match ghost_value {
                    Option::Some(s) => s,
                    _ => { proof_from_false() },
                },
            );
        }
    }
}

struct VEqualG {}

impl AtomicInvariantPredicate<(), u64, u64> for VEqualG {
    closed spec fn atomic_inv(k: (), v: u64, g: u64) -> bool {
        v == g
    }
}

proof fn proof_int(x: u64) -> (tracked y: u64)
    ensures
        x == y,
{
    assume(false);
    proof_from_false()
}

pub fn main() {
    // (Example usage of the atomic operations was omitted)
}

}

// Step 3 (inv_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1