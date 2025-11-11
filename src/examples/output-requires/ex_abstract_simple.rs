// Example: When to use ABSTRACT postconditions (simple cases)
// Shows standard operations where abstract view @ works perfectly

use vstd::prelude::*;

verus! {

pub struct SimpleList<T> {
    data: Vec<T>,
}

impl<T> SimpleList<T> {
    spec fn view(&self) -> Seq<T> {
        self.data@
    }

    // ========== ABSTRACT POSTCONDITION (CORRECT for simple case) ==========
    fn length(&self) -> (len: usize)
        ensures
            len == self@.len()  // ABSTRACT - simple and clear
    {
        self.data.len()
    }

    // ========== ABSTRACT POSTCONDITION (CORRECT for direct access) ==========
    fn get(&self, index: usize) -> (elem: &T)
        requires
            index < self@.len()
        ensures
            *elem == self@[index as int]  // ABSTRACT - natural and provable
    {
        &self.data[index]
    }

    // ========== ABSTRACT POSTCONDITION (CORRECT for standard update) ==========
    fn set(&mut self, index: usize, value: T)
        requires
            index < old(self)@.len()
        ensures
            self@ == old(self)@.update(index as int, value)  // ABSTRACT - clean
    {
        self.data.set(index, value);
    }
}

// ========== WHEN TO USE ABSTRACT POSTCONDITIONS ==========
//
// Use abstract view @ when:
// 1. Simple properties (length, equality)
// 2. Direct view mapping (no encoding/packing)
// 3. Standard operations (get, set, push, pop)
// 4. NO low-level proof functions involved
//
// These cases are EASY - abstract is natural and works!
//
// ==================================

} // verus!

fn main() {}
