use vstd::prelude::*;

verus! {

    // Original non-test code with Verus specs added
    pub fn main()
        ensures
            // main terminates normally
            true,
    {
    }

    /* TEST CODE BELOW */
    // Replacing the old Rust unit tests with Verus proof tests.
    pub fn test_main_runs_without_panicking()
        ensures
            // The test succeeds if main() executes without panicking.
            true,
    {
        // Calling main should not cause any panic.
        main();
    }
}