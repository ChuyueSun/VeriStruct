#![allow(unused_imports)]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::marker::PhantomData;

pub struct Lock<T> {
    field: AtomicBool,
    _marker: PhantomData<T>,
}

pub fn main() {
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atomic_operations() {
        let ato = AtomicU64::new(10u64);
        ato.fetch_or(19u64, Ordering::SeqCst);
        let _ = ato.fetch_or(23u64, Ordering::SeqCst);
        let _ = ato.compare_exchange(20u64, 25u64, Ordering::SeqCst, Ordering::SeqCst);
        let _ = ato.load(Ordering::SeqCst);
        ato.store(36u64, Ordering::SeqCst);
    }
}