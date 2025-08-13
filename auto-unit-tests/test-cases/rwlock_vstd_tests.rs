#![allow(unused_imports)]

struct FixedParity {
    pub parity: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_parity_positive() {
        let fixed = FixedParity { parity: 42 };
        assert_eq!(fixed.parity, 42);
    }

    #[test]
    fn test_fixed_parity_zero() {
        let fixed = FixedParity { parity: 0 };
        assert_eq!(fixed.parity, 0);
    }

    #[test]
    fn test_fixed_parity_negative() {
        let fixed = FixedParity { parity: -10 };
        assert_eq!(fixed.parity, -10);
    }

    #[test]
    fn test_fixed_parity_i64_max() {
        let fixed = FixedParity { parity: i64::MAX };
        assert_eq!(fixed.parity, i64::MAX);
    }

    #[test]
    fn test_fixed_parity_i64_min() {
        let fixed = FixedParity { parity: -9223372036854775808 };
        assert_eq!(fixed.parity, -9223372036854775808);
    }
}