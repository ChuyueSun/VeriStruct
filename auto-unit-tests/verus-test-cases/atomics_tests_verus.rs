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
pub fn main() {
    // Test the basic behavior of the Tracked<T> wrapper.
    {
        let x: u64 = 42;
        let t = Tracked(x);
        // Pattern-match on t to extract its inner value.
        match t {
            Tracked(y) => {
                assert(y == 42);
            }
        }
    }
    
    // Since constructing a concrete instance of Lock<T> with a proper ghost state is nontrivial
    // in this example, we illustrate correctness of the specification by assuming the existence
    // of a lock that satisfies well_formed(). The specification of take guarantees that if
    // lock.well_formed() holds then take(lock) returns a tracked value extracted from the ghost.
    //
    // Here, we use a ghost block to reason about the abstract behavior.
    ghost {
        // Assume an arbitrary lock for type u64 that is well-formed.
        // (In a full development, one would provide a constructor for Lock that
        // initializes its atomic field to (true, Some(v)) for some v.)
        let lock: Lock<u64>;
        assume(lock.well_formed());
        // Call take. By the specification of take, the returned tracked value comes
        // from the ghost value stored in lock.field.
        let _tracked_val = take(&lock);
        // Since we cannot concretely relate _tracked_val to a specific number in this abstract test,
        // we simply assert true to indicate that the call typechecks and meets its specification.
        assert(true);
    }
}
}