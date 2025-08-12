#![allow(unused_imports)]

struct ModPredicate {}

impl ModPredicate {
    fn inv(k: i32, v: u32) -> bool {
        v as i32 % 2 == k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_numbers_with_correct_k() {
        // For even numbers, v as i32 % 2 should be 0.
        assert!(ModPredicate::inv(0, 0));
        assert!(ModPredicate::inv(0, 2));
        assert!(ModPredicate::inv(0, 100));
    }

    #[test]
    fn test_even_numbers_with_incorrect_k() {
        // Even numbers with an incorrect k value (should not match).
        assert!(!ModPredicate::inv(1, 0));
        assert!(!ModPredicate::inv(1, 2));
        assert!(!ModPredicate::inv(-1, 100));
    }

    #[test]
    fn test_odd_numbers_with_correct_k() {
        // For odd numbers, v as i32 % 2 should be 1.
        assert!(ModPredicate::inv(1, 1));
        assert!(ModPredicate::inv(1, 3));
        assert!(ModPredicate::inv(1, 101));
    }

    #[test]
    fn test_odd_numbers_with_incorrect_k() {
        // Odd numbers with an incorrect k value (should not match).
        assert!(!ModPredicate::inv(0, 1));
        assert!(!ModPredicate::inv(0, 3));
        assert!(!ModPredicate::inv(-1, 101));
    }

    #[test]
    fn test_boundary_values_and_overflow_casting() {
        // u32::MAX when cast to i32 gives -1.
        assert!(ModPredicate::inv(-1, u32::MAX));
        // Another large value: 4294967294 as u32 is 4294967294 - 2*2147483648 = -2 as i32.
        // -2 % 2 equals 0.
        assert!(ModPredicate::inv(0, 4294967294));
        assert!(!ModPredicate::inv(-1, 4294967294));

        // Test with 2147483647 (i32::MAX) which should remain positive.
        // 2147483647 % 2 = 1.
        assert!(ModPredicate::inv(1, 2147483647));

        // Test with 2147483648 which when cast becomes -2147483648.
        // -2147483648 % 2 equals 0.
        assert!(ModPredicate::inv(0, 2147483648));
    }

    #[test]
    fn test_incorrect_k_values() {
        // Test with deliberately wrong k values.
        // Even number with wrong k:
        assert!(!ModPredicate::inv(2, 4));
        // Odd number with wrong k:
        assert!(!ModPredicate::inv(2, 3));
        // u32::MAX only matches k = -1.
        assert!(!ModPredicate::inv(0, u32::MAX));
        assert!(!ModPredicate::inv(1, u32::MAX));
    }
}