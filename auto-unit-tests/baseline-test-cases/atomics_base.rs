#![allow(unused_imports)]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::cell::UnsafeCell;

struct Lock<T> {
    field: AtomicBool,
    value: UnsafeCell<Option<T>>,
}

impl<T> Lock<T> {
    fn new(val: T) -> Self {
        Lock {
            field: AtomicBool::new(true),
            value: UnsafeCell::new(Some(val)),
        }
    }
}

fn take<T>(lock: &Lock<T>) -> T {
    loop {
        if lock.field.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            unsafe {
                if let Some(val) = (*lock.value.get()).take() {
                    return val;
                } else {
                    panic!("Invariant broken");
                }
            }
        }
    }
}

fn proof_int(x: u64) -> u64 {
    x
}

/* TEST CODE BELOW */

pub fn test() {

    let ato = AtomicU64::new(10u64);

    ato.fetch_or(19u64, Ordering::SeqCst);

    {
        let old_val = ato.load(Ordering::SeqCst);
        let _ = ato.fetch_or(23u64, Ordering::SeqCst);
        let new_val = ato.load(Ordering::SeqCst);
        assert_eq!(new_val, old_val | 23u64);
    }
    
    {
        let old_val = ato.load(Ordering::SeqCst);
        let res = ato.compare_exchange(20u64, 25u64, Ordering::SeqCst, Ordering::SeqCst);
        let new_val = ato.load(Ordering::SeqCst);
        match res {
            Ok(_) => {
                assert!(old_val == 20u64 && new_val == 25u64);
            },
            Err(e) => {
                assert!(old_val != 20u64 && new_val == old_val && e == old_val);
            }
        }
    }
    
    let _res = ato.load(Ordering::SeqCst);
    
    {
        let _old_val = ato.load(Ordering::SeqCst);
        ato.store(36u64, Ordering::SeqCst);
        let new_val = ato.load(Ordering::SeqCst);
        assert_eq!(new_val, 36u64);
    }
}

pub fn main() {
}