#![allow(unused_imports)]

use std::sync::RwLock;

pub fn main() {
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;

    #[test]
    fn test_rwlock() {
        let lock_even = RwLock::new(20);
        let lock_odd = RwLock::new(23);

        {
            let read_handle_even = lock_even.read().unwrap();
            let val_even = *read_handle_even;
            assert!(val_even % 2 == 0);
        }
        {
            let read_handle_odd = lock_odd.read().unwrap();
            let val_odd = *read_handle_odd;
            assert!(val_odd % 2 == 1);
        }

        let n: u64 = 42;
        let lock_arbitrary = RwLock::new(n);
        let read_handle_arbitrary = lock_arbitrary.read().unwrap();
        let val_arbitrary = *read_handle_arbitrary;
        assert!(val_arbitrary % 2 == n % 2);
    }
}