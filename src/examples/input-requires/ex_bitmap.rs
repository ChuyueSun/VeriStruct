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

    fn create(v: Vec<SomeType>) -> (ret: DataStructure)
    // TODO: add requires and ensures
    {
        DataStructure { data: v }
    }

    fn get_element(&self, index: u32) -> (elem: ElementType)
    // TODO: add requires and ensures
    {
        // ... implementation using self.data[index] ...
    }

    fn update_element(&mut self, index: u32, value: ElementType)
    // TODO: add requires and ensures
    {
        // ... implementation using self.data.set(index, value) ...
    }
}

} // verus!
