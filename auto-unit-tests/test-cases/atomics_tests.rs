#![allow(unused_imports)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::UnsafeCell;

pub struct Lock<T> {
    flag: AtomicBool,
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send> Sync for Lock<T> {}

impl<T> Lock<T> {
    pub fn new(val: T) -> Self {
        Lock {
            flag: AtomicBool::new(true),
            value: UnsafeCell::new(Some(val)),
        }
    }
}

pub fn take<T>(lock: &Lock<T>) -> T {
    loop {
        if lock.flag.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            unsafe {
                if let Some(val) = (*lock.value.get()).take() {
                    return val;
                } else {
                    panic!("Invariant violated: flag is true but no value");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_take_i32() {
        let lock = Lock::new(42);
        let val = take(&lock);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_take_string() {
        let lock = Lock::new(String::from("hello"));
        let val = take(&lock);
        assert_eq!(val, "hello");
    }

    #[test]
    #[should_panic(expected = "Invariant violated: flag is true but no value")]
    fn test_invariant_panic() {
        let lock = Lock::new(99);
        // Manually violate the invariant: set the inner value to None while leaving the flag as true.
        unsafe {
            *lock.value.get() = None;
        }
        let _ = take(&lock);
    }

    // Test basic concurrent behavior: only one thread should successfully take the value.
    #[test]
    fn test_take_concurrent_single_consumer() {
        let lock = Lock::new(100);
        // Create a raw pointer to 'lock' which will remain valid for the duration of this test.
        let lock_ptr = &lock as *const Lock<_>;
        // Spawn a thread to take the value from the lock.
        let handle = thread::spawn(move || {
            // SAFETY: 'lock' is not moved and the pointer remains valid.
            let lock_ref = unsafe { &*lock_ptr };
            take(lock_ref)
        });
        // Wait a short duration to give the spawned thread a chance to run.
        thread::sleep(Duration::from_millis(10));
        // The spawned thread should have taken the value.
        let val = handle.join().unwrap();
        assert_eq!(val, 100);
    }
}