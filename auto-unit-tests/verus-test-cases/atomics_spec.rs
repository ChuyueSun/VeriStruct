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
        invariant on field with () is (b: bool, t: Option<T>) {
            b === t.is_Some()
        }
    }
}

#[verifier::exec_allows_no_decreases_clause]
fn take<T>(lock: &Lock<T>) -> (t: Tracked<T>)
    requires
        lock.well_formed(),
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
        v === g
    }
}

proof fn proof_int(x: u64) -> (tracked y: u64)
    ensures
        x == y,
{
    assume(false);
    proof_from_false()
}


/* TEST CODE BELOW */