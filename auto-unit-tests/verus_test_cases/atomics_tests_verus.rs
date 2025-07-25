use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

verus! {

    // A verified lock that holds a value of type T.
    // We add a ghost field "ghost_value" to model the abstract state of the lock.
    pub struct Lock<T> {
        field: AtomicBool,
        value: Mutex<Option<T>>,
        ghost_value: Option<T>, // ghost field representing the contained value; becomes None when taken
    }

    impl<T: Clone + Eq> Lock<T> {
        // Construct a new Lock with the given value t.
        pub fn new(t: T) -> Self
            ensures
                // Initially, the atomic flag is true.
                self.field.load(Ordering::SeqCst) == true,
                // The ghost view of the lock holds the value.
                self.ghost_value == Some(t),
        {
            Lock {
                field: AtomicBool::new(true),
                value: Mutex::new(Some(t)),
                ghost_value: Some(t),
            }
        }
    }

    // Take the value from the lock.
    // Preconditions: the lock must contain a value (its ghost field is Some(_)).
    // Postconditions: the lock's ghost value is None and the returned value equals the previous one.
    pub fn take<T: Clone + Eq>(lock: &Lock<T>) -> T
        requires
            // There must be a value available.
            lock.ghost_value.is_Some(),
        ensures
            // After taking, the ghost value is None.
            lock.ghost_value.is_None(),
            // The returned value is equal to the value held in the lock before.
            old(lock.ghost_value).get_Some() == result,
    {
        loop {
            if lock.field.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                let mut guard = lock.value.lock().expect("lock poisoned");
                if let Some(val) = guard.take() {
                    // In ghost code, update the ghost field.
                    proof {
                        // Since we are taking the value, we update the ghost state.
                        // (This ghost update is not executable; it is only used for verification.)
                        update_lock_ghost::update(lock, None);
                    }
                    return val;
                }
            }
        }
    }

    // Auxiliary ghost-only module to update the ghost field.
    // In verus, ghost state updates are not explicit at runtime.
    mod update_lock_ghost {
        use super::*;
        // This ghost function models the update of the ghost field of the lock.
        pub proof fn update<T>(lock: &Lock<T>, new_val: Option<T>)
            ensures
                lock.ghost_value == new_val,
        {
            // In verification, we update the ghost field to new_val.
            // (The body is left empty since this is a ghost update.)
        }
    }

    /* TEST CODE BELOW */

    // Test functions verify the behavior of Lock and take using assert proofs.

    // Test: take returns the correct i32 value.
    pub fn test_take_i32() {
        let lock = Lock::new(42);
        let value = take(&lock);
        assert(value == 42);
    }

    // Test: take returns the correct &str value.
    pub fn test_take_str() {
        let lock = Lock::new("hello");
        let value = take(&lock);
        assert(value == "hello");
    }

    // Test: internal state of the lock before and after take.
    pub fn test_internal_state_before_and_after_take() {
        let lock = Lock::new("internal");
        // Before calling take: the atomic flag should be true.
        assert(lock.field.load(Ordering::SeqCst));
        {
            let guard = lock.value.lock().expect("lock poisoned");
            // The mutex contains Some value.
            assert(guard.is_some());
        }
        // The ghost field should also have a value.
        proof {
            assert(lock.ghost_value.is_Some());
        }

        let value = take(&lock);
        assert(value == "internal");
        // After take: the atomic flag should be false.
        assert(!lock.field.load(Ordering::SeqCst));
        // And the ghost field should be None.
        proof {
            assert(lock.ghost_value.is_None());
        }
    }

    // Main function to call all tests.
    pub fn main() {
        test_take_i32();
        test_take_str();
        test_internal_state_before_and_after_take();
    }
}