use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

pub struct Lock<T> {
    field: AtomicBool,
    value: Mutex<Option<T>>,
}

impl<T> Lock<T> {
    pub fn new(t: T) -> Self {
        Lock {
            field: AtomicBool::new(true),
            value: Mutex::new(Some(t)),
        }
    }
}

pub fn take<T>(lock: &Lock<T>) -> T {
    loop {
        if lock.field.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            let mut guard = lock.value.lock().unwrap();
            if let Some(val) = guard.take() {
                return val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_take_i32() {
        let lock = Lock::new(42);
        let value = take(&lock);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_take_str() {
        let lock = Lock::new("hello");
        let value = take(&lock);
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_internal_state_before_and_after_take() {
        let lock = Lock::new("internal");
        // Before calling take, the atomic field should be true and the mutex should contain Some(value).
        assert!(lock.field.load(Ordering::SeqCst));
        {
            let guard = lock.value.lock().unwrap();
            assert!(guard.is_some());
        }

        let value = take(&lock);
        assert_eq!(value, "internal");
        
        // After take, the atomic field should be false and the mutex