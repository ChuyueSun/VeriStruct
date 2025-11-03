use vstd::prelude::*;

verus! {

proof fn combine_proof(item1: ItemType, item2: ItemType, result: ItemType)
    requires
        result == combine_items(item1, item2),
    ensures
        // ... properties about the combined result ...
{
}

pub struct Container {
    items: Vec<ItemType>,
}

impl Container {
    spec fn view(&self) -> Seq<ViewType> {
        // ... converts items to view representation ...
    }

    fn combine(&self, other: &Container) -> (ret: Container)
        requires
            self@.len() == other@.len(),
        ensures
            ret@.len() == self@.len(),
            forall|i: int| #![auto] 0 <= i < ret@.len() ==>
                ret@[i] == combine_operation(self@[i], other@[i]),
    {
        let n: usize = self.items.len();
        let mut i: usize = 0;
        let mut result_items: Vec<ItemType> = Vec::new();
        let mut result = Container { items: result_items };
        while i < n
            // ========== INFERRED INVARIANTS ==========
            invariant
                i <= n,
                // CRITICAL: Connect loop bound to actual vector lengths
                n == self.items@.len(),
                n == other.items@.len(),
                i == result.items.len(),
                // CRITICAL: State the property at abstract (view) level
                forall|k: int| #![auto] 0 <= k < result@.len() ==>
                    result@[k] == combine_operation(self@[k], other@[k]),
            // =========================================
        {
            result_items = result.items;
            let item1: ItemType = self.items[i];
            let item2: ItemType = other.items[i];
            let combined: ItemType = combine_items(item1, item2);
            // ========== INFERRED PROOF ==========
            proof {
                combine_proof(item1, item2, combined);
                // Keep proof blocks simple - just call the proof function
                // The loop invariant does most of the work
            }
            // ====================================
            result_items.push(combined);
            result = Container { items: result_items };
            i = i + 1;
        }
        result
    }
}

} // verus!
