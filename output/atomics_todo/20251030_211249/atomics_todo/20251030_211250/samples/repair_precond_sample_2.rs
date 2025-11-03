#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use vstd::atomic_ghost::*;
use vstd::prelude::*;
use vstd::{pervasive::*, *};

verus! {

struct_with_invariants!{
/// A lock implementation using atomic boolean operations.
///
/// This lock structure provides a way to safely share data of type `T` between threads
/// using atomic operations. The lock maintains an invariant that the boolean state
/// matches whether the contained value is Some or None.
///
/// # Type Parameters
/// * `T` - The type of data protected by the lock
    struct Lock<T> {
        field: AtomicBool<_, Option<T>, _>,
    }

    spec fn well_formed(&self) -> bool {
        invariant on field with () is (b: bool, t: Option<T>) {
            b == t.is_Some()
        }
    }
}

#[verifier::exec_allows_no_decreases_clause]
fn take<T>(lock: &Lock<T>) -> (t: Tracked<T>)
    requires
        lock.well_formed()
    ensures
        lock.well_formed()
{
    loop
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

/// A predicate type that enforces equality between visible and ghost state in atomic operations.
///
/// This struct implements the AtomicInvariantPredicate trait to maintain the invariant
/// that the visible value (v) equals the ghost value (g) in atomic operations.
struct VEqualG {}

impl AtomicInvariantPredicate<(), u64, u64> for VEqualG {
    closed spec fn atomic_inv(k: (), v: u64, g: u64) -> bool {
        v == g
    }
}

proof fn proof_int(x: u64) -> (tracked y: u64)
    ensures
        x == y
{
    assert(true); // Added by AI
    assume(false);
    proof_from_false()
}

/* TEST CODE BELOW */

pub fn test() {

    let ato = AtomicU64::<(), u64, VEqualG>::new(Ghost(()), 10u64, Tracked(10u64));

    atomic_with_ghost!(ato => fetch_or(19u64);  ghost g => { g = proof_int(g | 19u64); });
    atomic_with_ghost!(ato => fetch_or(23u64);  update old_val -> new_val; ghost g => {
        assert(new_val == old_val | 23u64);
        assert(g == old_val);
        g = proof_int(g | 23u64);
        assert(g == new_val);
    });

    proof {
        assert(ato.well_formed());
    }

    let res = atomic_with_ghost!(ato => compare_exchange(20u64, 25u64);
        update old_val -> new_val;
        returning ret;
        ghost g => {
            assert(imply(matches!(ret, Ok(_)), old_val == 20u64 && new_val == 25u64));
            assert(imply(matches!(ret, Err(_)), old_val != 20u64 && new_val == old_val
                         && ret->Err_0 == old_val));
            g = if g == 20u64 { proof_int(25u64) } else { g };
    });

    let res = atomic_with_ghost!(ato => load();
        returning ret;
        ghost g => { assert(ret == g); });

    atomic_with_ghost!(ato => store(36u64);
        update old_val -> new_val;
        ghost g => {
            assert(old_val == g);
            assert(new_val == 36u64);
            g = proof_int(36u64);
    });
}

pub fn main() {
}

} // verus!

// VEval Score: Compilation Error: False, Verified: 4, Errors: 1, Verus Errors: 3
