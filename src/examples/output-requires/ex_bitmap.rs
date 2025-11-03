// Example: Custom data structure with view function
// Shows how to specify requires/ensures for types with view()

use vstd::prelude::*;

verus! {

pub struct DataStructure {
    data: Vec<SomeType>,
}

impl DataStructure {
    // When a type has spec fn view() -> Seq<T>, use @ for the view
    spec fn view(&self) -> Seq<ElementType> {
        // ... implementation ...
    }

    // Constructor pattern: relate return value's view to input
    fn create(v: Vec<SomeType>) -> (ret: DataStructure)
        // ========== INFERRED SPECIFICATIONS ==========
        ensures
            ret@.len() == some_function_of(v),  // Use ret@ not ret.view()
        // =============================================
    {
        DataStructure { data: v }
    }

    // Getter pattern: bound check and correctness
    fn get_element(&self, index: u32) -> (elem: ElementType)
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            index < self@.len(),  // Use self@ not self.view()
        ensures
            elem == self@[index as int],  // Use self@ not self.view()
        // =============================================
    {
        // ... implementation using self.data[index] ...
    }

    // Setter pattern: use .update() in postcondition
    fn update_element(&mut self, index: u32, value: ElementType)
        // ========== INFERRED SPECIFICATIONS ==========
        requires
            index < old(self)@.len(),  // Use old(self)@ not old(self).view()
        ensures
            self@ == old(self)@.update(index as int, value),  // Use @ and .update()
        // =============================================
    {
        // ... implementation using self.data.set(index, value) ...
    }
}

} // verus!
