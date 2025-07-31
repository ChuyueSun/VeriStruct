struct FixedParity {
    pub parity: i64,
}

use std::sync::RwLock;

fn main() {}

fn test(n: u64) {
    let lock_even = RwLock::new(20);
    let lock_odd = RwLock::new(23);

    let read_handle_even = lock_even.read().unwrap();
    let val_even = *read_handle_even;
    assert!(val_even % 2 == 0);

    let read_handle_odd = lock_odd.read().unwrap();
    let val_odd = *read_handle_odd;
    assert!(val_odd % 2 == 1);

    let lock_arbitrary = RwLock::new(n);
    let read_handle_arbitrary = lock_arbitrary.read().unwrap();
    let val_arbitrary = *read_handle_arbitrary;
    assert!(val_arbitrary % 2 == n % 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_even_number() {
        // Test with a typical even number.
        test(100);
    }

    #[test]
    fn test_with_odd_number() {
        // Test with a typical odd number.
        test(101);
    }

    #[test]
    fn test_with_zero() {
        // Zero is even.
        test(0);
    }

    #[test]
    fn test_with_u64_max() {
        // u64::MAX is 18446744073709551615, which is odd.
        test(u64::MAX);
    }

    #[test]
    fn test_main_function() {
        // Ensure that the main function, even though empty, is callable.
        main();
    }
}