#![allow(unused_imports)]

struct FixedParity {
    pub parity: i64,
}

impl FixedParity {
    fn normalized_parity(&self) -> i64 {
        // Use the Euclidean remainder so that the result is always nonnegative.
        self.parity.rem_euclid(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_even() {
        let fp = FixedParity { parity: 2 };
        // Check that a positive even number has an even parity.
        assert_eq!(fp.normalized_parity(), 0);
    }

    #[test]
    fn test_positive_odd() {
        let fp = FixedParity { parity: 3 };
        // Check that a positive odd number has an odd parity.
        assert_eq!(fp.normalized_parity(), 1);
    }

    #[test]
    fn test_zero() {
        let fp = FixedParity { parity: 0 };
        // Zero is considered even.
        assert_eq!(fp.normalized_parity(), 0);
    }

    #[test]
    fn test_negative_even() {
        let fp = FixedParity { parity: -4 };
        // Check that a negative even number remains even.
        assert_eq!(fp.normalized_parity(), 0);
    }

    #[test]
    fn test_negative_odd() {
        let fp = FixedParity { parity: -5 };
        // Check that a negative odd number remains odd.
        // Using normalized_parity to consistently get a nonnegative result.
        assert_eq!(fp.normalized_parity(), 1);
    }

    #[test]
    fn test_max_value() {
        let fp = FixedParity { parity: i64::MAX };
        // i64::MAX is 9223372036854775807 which is odd.
        assert_eq!(fp.normalized_parity(), 1);
    }

    #[test]
    fn test_min_value() {
        let fp = FixedParity { parity: i64::MIN };
        // i64::MIN is -9223372036854775808 which is even.
        assert_eq!(fp.normalized_parity(), 0);
    }
}