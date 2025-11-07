// Example: Direct comparison of ABSTRACT vs CONCRETE approaches
// Shows the SAME operation with both abstraction levels and when each works

use vstd::prelude::*;

verus! {

// ========== SCENARIO 1: Simple Structure (ABSTRACT works) ==========

pub struct SimpleContainer<T> {
    items: Vec<T>,
}

impl<T> SimpleContainer<T> {
    spec fn view(&self) -> Seq<T> {
        self.items@  // Direct mapping - no encoding
    }

    // ABSTRACT postcondition - WORKS because no encoding/proofs
    fn merge(&self, other: &SimpleContainer<T>) -> (result: SimpleContainer<T>)
        requires
            self@.len() == other@.len()
        ensures
            result@.len() == self@.len(),
            // ABSTRACT is FINE here - direct semantic property
            forall|i: int| #![auto] 0 <= i < result@.len() ==>
                result@[i] == if some_condition(i) { self@[i] } else { other@[i] }
    {
        // ... implementation without low-level proofs ...
    }
}

// ========== SCENARIO 2: Packed Structure (CONCRETE required) ==========

proof fn packed_combine_proof(unit1: u64, unit2: u64, result: u64)
    requires
        result == combine_at_unit_level(unit1, unit2)
    ensures
        // Proof operates at UNIT level (u64), not logical element level
        forall|elem_idx: u64| #![auto] elem_idx < ELEMENTS_PER_UNIT ==>
            get_element_from_unit(result, elem_idx) ==
            merge_elements(
                get_element_from_unit(unit1, elem_idx),
                get_element_from_unit(unit2, elem_idx)
            )
{
}

pub struct PackedContainer {
    units: Vec<u64>,  // Packed - multiple logical elements per u64
}

impl PackedContainer {
    spec fn view(&self) -> Seq<LogicalElement> {
        // View EXPANDS units to logical elements
        Seq::new(self.units@.len() * ELEMENTS_PER_UNIT, |i: int| {
            get_element_from_unit(self.units@[i / ELEMENTS_PER_UNIT], (i % ELEMENTS_PER_UNIT) as u64)
        })
    }

    // ❌ WRONG - Abstract postcondition (UNPROVABLE with packed_combine_proof!)
    /*
    fn merge_wrong(&self, other: &PackedContainer) -> (result: PackedContainer)
        ensures
            forall|i: int| result@[i] == merge_elements(self@[i], other@[i])
            //                ^^^^^^^^^ UNPROVABLE!
            // Why: packed_combine_proof talks about units, not logical elements
            // No connection between proof and this postcondition!
    */

    // ✅ CORRECT - Concrete postcondition (PROVABLE!)
    fn merge_correct(&self, other: &PackedContainer) -> (result: PackedContainer)
        requires
            self.units@.len() == other.units@.len()
        ensures
            result.units@.len() == self.units@.len(),
            // CONCRETE: Reference units directly (matches proof level!)
            forall|i: int| #![auto] 0 <= i < result@.len() ==> {
                let unit_idx = i / ELEMENTS_PER_UNIT;
                let elem_idx = (i % ELEMENTS_PER_UNIT) as u64;
                get_element_from_unit(result.units@[unit_idx], elem_idx) ==
                merge_elements(
                    get_element_from_unit(self.units@[unit_idx], elem_idx),
                    get_element_from_unit(other.units@[unit_idx], elem_idx)
                )
            }
    {
        let mut result_units: Vec<u64> = Vec::new();
        let mut i: usize = 0;

        while i < self.units.len()
        {
            let u1 = self.units[i];
            let u2 = other.units[i];
            let combined = combine_at_unit_level(u1, u2);

            proof {
                packed_combine_proof(u1, u2, combined);
                // Proof establishes: get_element_from_unit(combined, idx) == merge(...)
                // Our postcondition uses: get_element_from_unit(result.units@[...], ...)
                // SAME LEVEL → Verus can connect them! ✓
            }

            result_units.push(combined);
            i = i + 1;
        }

        PackedContainer { units: result_units }
    }
}

// ========== THE CRITICAL DIFFERENCE ==========
//
// **Simple structure (SimpleContainer):**
// - items: Vec<T> → view: Seq<T>
// - Direct mapping, no encoding
// - Abstract postconditions WORK
// - Can use: result@[i] == ...
//
// **Packed structure (PackedContainer):**
// - units: Vec<u64> → view: Seq<LogicalElement>
// - Packed encoding (N elements per u64)
// - Proof operates on u64 chunks
// - Abstract postconditions DON'T WORK
// - MUST use: get_element_from_unit(result.units@[i/N], i%N) == ...
//
// **The Rule:**
// If proof function signature contains the UNDERLYING type (u64, chunks, units),
// postcondition MUST also reference that UNDERLYING type!
//
// ========================================

} // verus!

fn main() {}
