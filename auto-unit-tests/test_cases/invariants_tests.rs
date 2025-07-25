#![allow(unused_imports)]

pub fn main() {
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
    assert!(j % 2 == 1);
}

pub fn if_branch(i: u32) -> u32 {
    let mut i = i;
    if i == 1u32 {
        i = 3u32;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_main_runs_without_panic() {
        // Calling main should not cause any panic.
        main();
    }

    #[test]
    fn test_main_does_not_panic_via_catch_unwind() {
        // Ensure main executes successfully by using catch_unwind.
        let result = panic::catch_unwind(|| {
            main();
        });
        assert!(result.is_ok(), "main() panicked unexpectedly");
    }

    #[test]
    fn test_main_multiple_invocations() {
        // Call main several times to check for any unexpected side effects.
        for _ in 0..10 {
            main();
        }
    }

    #[test]
    fn test_if_branch_changes_value() {
        // Test that the branch setting i = 3u32 is executed when i is 1.
        let result = if_branch(1);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_if_branch_does_not_change_value() {
        // Test that if_branch leaves i unchanged when i is not 1.
        let result = if_branch(5);
        assert_eq!(result, 5);
    }
}