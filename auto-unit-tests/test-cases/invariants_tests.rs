#![allow(unused_imports)]

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assuming the main module defines a function with this signature:
    // pub fn add(a: i32, b: i32) -> i32

    #[test]
    fn test_add_positive_numbers() {
        // Typical case: two positive numbers.
        let result = add(2, 3);
        assert_eq!(result, 5, "Expected add(2, 3) to yield 5, got {}", result);
    }

    #[test]
    fn test_add_negative_numbers() {
        // Typical case: two negative numbers.
        let result = add(-4, -6);
        assert_eq!(result, -10, "Expected add(-4, -6) to yield -10, got {}", result);
    }

    #[test]
    fn test_add_mixed_sign_numbers() {
        // Mixed signs, positive + negative.
        let result = add(-7, 10);
        assert_eq!(result, 3, "Expected add(-7, 10) to yield 3, got {}", result);
    }

    #[test]
    fn test_add_with_zero() {
        // Edge case: one or both zeros.
        let result1 = add(0, 5);
        let result2 = add(5, 0);
        let result3 = add(0, 0);
        assert_eq!(result1, 5, "Expected add(0, 5) to yield 5, got {}", result1);
        assert_eq!(result2, 5, "Expected add(5, 0) to yield 5, got {}", result2);
        assert_eq!(result3, 0, "Expected add(0, 0) to yield 0, got {}", result3);
    }

    #[test]
    fn test_add_large_numbers() {
        // Edge case: large integer values.
        let a = 1_000_000;
        let b = 2_000_000;
        let result = add(a, b);
        assert_eq!(result, 3_000_000, "Expected add({}, {}) to yield 3000000, got {}", a, b, result);
    }
}