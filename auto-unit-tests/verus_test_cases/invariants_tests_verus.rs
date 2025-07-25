#![allow(unused_imports)]
use vstd::prelude::*;

verus! {

    pub fn main()
        ensures
            true, // No explicit postcondition, the internal assert verifies the invariant.
    {
        let u: u32 = 5u32;
        let mut i = u;
        if i == 1u32 {
            i = 3u32;
        }
        let mut j = 7u32;
        {
            let tmp = i;
            i = j;
            j = tmp;
        }
        let j = i;
        assert(j % 2 == 1);
    }

    pub fn if_branch(i: u32) -> (result: u32)
        ensures
            if i == 1u32 { result == 3u32 } else { result == i },
    {
        let mut i = i;
        if i == 1u32 {
            i = 3u32;
        }
        i
    }

    /* TEST CODE BELOW */
    pub fn test_main_runs_without_panic() {
        // Calling main should not cause any panic.
        main();
    }

    pub fn test_main_does_not_panic_via_catch_unwind() {
        // In Verus, if main were to panic, verification would fail.
        // Thus, a simple call suffices as a proof that main terminates normally.
        main();
    }

    pub fn test_main_multiple_invocations() {
        // Call main several times to check for any unexpected side effects.
        let mut k: u32 = 0;
        while k < 10
            invariant
                k <= 10,
            decreases
                10 - k
        {
            main();
            k = k + 1;
        }
    }

    pub fn test_if_branch_changes_value() {
        // Test that the branch setting i = 3u32 is executed when i is 1.
        let result = if_branch(1);
        assert(result == 3u32);
    }

    pub fn test_if_branch_does_not_change_value() {
        // Test that if_branch leaves i unchanged when i is not 1.
        let result = if_branch(5);
        assert(result == 5u32);
    }

    pub fn verus_all_tests() {
        test_main_runs_without_panic();
        test_main_does_not_panic_via_catch_unwind();
        test_main_multiple_invocations();
        test_if_branch_changes_value();
        test_if_branch_does_not_change_value();
    }
}