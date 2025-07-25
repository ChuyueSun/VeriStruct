#![allow(unused_imports)]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct Lock<T> {
    pub field: AtomicBool,
}

pub fn take<T>(_lock: &Lock<T>) -> T {
    unimplemented!()
}

pub fn main() {

    let ato = AtomicU64::new(10u64);

    ato.fetch_or(19u64, Ordering::SeqCst);

    let old_val = ato.fetch_or(23u64, Ordering::SeqCst);
    let new_val = ato.load(Ordering::SeqCst);
    assert_eq!(new_val, old_val | 23u64);
    
    let res = ato.compare_exchange(20u64, 25u64, Ordering::SeqCst, Ordering::SeqCst);
    
    let ret = ato.load(Ordering::SeqCst);
    
    ato.store(36u64, Ordering::SeqCst);
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering, AtomicBool};

    #[test]
    fn test_main_runs_without_panic() {
        // Calling main to ensure it runs without panicking.
        main();
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_take_panics() {
        // Create a dummy Lock instance for testing.
        let dummy_lock: Lock<u64> = Lock { field: AtomicBool::new(true) };
        // Calling take is expected to panic since it's unimplemented.
        let _ = take(&dummy_lock);
    }

    #[test]
    fn test_atomic_fetch_or_operation() {
        // Replicate the atomic fetch_or operation as in main.
        let ato = AtomicU64::new(10u64);
        // First fetch_or with 19.
        ato.fetch_or(19u64, Ordering::SeqCst);
        // Second fetch_or with 23; capture old value.
        let old_val = ato.fetch_or(23u64, Ordering::SeqCst);
        let new_val = ato.load(Ordering::SeqCst);
        // new_val should equal old_val OR 23.
        assert_eq!(new_val, old_val | 23u64);
    }

    #[test]
    fn test_atomic_compare_exchange_success() {
        // Initialize with the expected value.
        let ato = AtomicU64::new(20u64);
        // This compare_exchange should succeed.
        let result = ato.compare_exchange(20u64, 25u64, Ordering::SeqCst, Ordering::SeqCst);
        // Expect Ok with the previous value.
        assert_eq!(result, Ok(20u64));
        // The atomic value should now be updated.
        assert_eq!(ato.load(Ordering::SeqCst), 25u64);
    }

    #[test]
    fn test_atomic_compare_exchange_failure() {
        // Initialize with a value that does not match expected.
        let ato = AtomicU64::new(30u64);
        // This compare_exchange should fail.
        let result = ato.compare_exchange(20u64, 25u64, Ordering::SeqCst, Ordering::SeqCst);
        // Expect Err with the current value.
        assert_eq!(result, Err(30u64));
        // The atomic value remains unchanged.
        assert_eq!(ato.load(Ordering::SeqCst), 30u64);
    }

    #[test]
    fn test_atomic_store_and_load() {
        let ato = AtomicU64::new(0u64);
        // Store a value.
        ato.store(36u64, Ordering::SeqCst);
        // Load and check the value.
        let ret = ato.load(Ordering::SeqCst);
        assert_eq!(ret, 36u64);
    }
}