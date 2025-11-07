// Example: Loop Invariants and Proofs with Abstraction Level Selection
// Shows when to use ABSTRACT vs CONCRETE level in loop invariants and postconditions

use vstd::prelude::*;

verus! {

// ========== EXAMPLE 1: ABSTRACT LEVEL (Simple Operations) ==========

proof fn combine_proof(item1: ItemType, item2: ItemType, result: ItemType)
    requires
        result == combine_items(item1, item2),
    ensures
        property_about_result(result, item1, item2)
{
}

pub struct Container {
    items: Vec<ItemType>,
}

impl Container {
    spec fn view(&self) -> Seq<ViewType> {
        self.items@.map(|i, item| convert_to_view(item))
    }

    // Use ABSTRACT level when: No low-level proof functions involved
    fn combine_abstract(&self, other: &Container) -> (ret: Container)
        requires
            self@.len() == other@.len(),
        ensures
            ret@.len() == self@.len(),
            // ABSTRACT postcondition - works for high-level operations
            forall|i: int| #![auto] 0 <= i < ret@.len() ==>
                ret@[i] == combine_operation(self@[i], other@[i]),
    {
        let n: usize = self.items.len();
        let mut i: usize = 0;
        let mut result_items: Vec<ItemType> = Vec::new();
        let mut result = Container { items: result_items };
        while i < n
            invariant
                i <= n,
                n == self.items@.len(),
                n == other.items@.len(),
                i == result.items.len(),
                // ABSTRACT invariant - matches abstract postcondition
                forall|k: int| #![auto] 0 <= k < result@.len() ==>
                    result@[k] == combine_operation(self@[k], other@[k]),
        {
            result_items = result.items;
            let combined = combine_items(self.items[i], other.items[i]);

            proof {
                combine_proof(self.items[i], other.items[i], combined);
            }

            result_items.push(combined);
            result = Container { items: result_items };
            i = i + 1;
        }
        result
    }
}

// ========== EXAMPLE 2: CONCRETE LEVEL (Packed/Low-Level Operations) ==========

proof fn unit_combine_proof(unit1: UnderlyingUnit, unit2: UnderlyingUnit, result: UnderlyingUnit)
    requires
        result == combine_units(unit1, unit2),
    ensures
        // Proof establishes property at CONCRETE level (about components within units)
        forall|comp: ComponentIdx| #![auto] component_in_range(comp) ==>
            extract_from_unit(result, comp) ==
            combine_values(
                extract_from_unit(unit1, comp),
                extract_from_unit(unit2, comp)
            )
{
}

pub struct PackedContainer {
    units: Vec<UnderlyingUnit>,  // Packed/encoded storage
}

impl PackedContainer {
    spec fn view(&self) -> Seq<LogicalValue> {
        // View unpacks units into logical sequence
        Seq::new(self.units@.len() * COMPONENTS_PER_UNIT, |i: int| {
            let unit_idx = i / COMPONENTS_PER_UNIT;
            let comp_idx = (i % COMPONENTS_PER_UNIT) as ComponentIdx;
            extract_from_unit(self.units@[unit_idx], comp_idx)
        })
    }

    // Use CONCRETE level when: Proof functions operate on UnderlyingUnit type
    fn combine_concrete(&self, other: &PackedContainer) -> (ret: PackedContainer)
        requires
            self.units@.len() == other.units@.len(),
        ensures
            ret.units@.len() == self.units@.len(),
            // CONCRETE postcondition - matches what unit_combine_proof establishes!
            forall|i: int| #![auto] 0 <= i < ret@.len() ==> {
                let unit_i = i / COMPONENTS_PER_UNIT;
                let comp_i = (i % COMPONENTS_PER_UNIT) as ComponentIdx;
                extract_from_unit(ret.units@[unit_i], comp_i) ==
                combine_values(
                    extract_from_unit(self.units@[unit_i], comp_i),
                    extract_from_unit(other.units@[unit_i], comp_i)
                )
            }
    {
        let n: usize = self.units.len();
        let mut i: usize = 0;
        let mut result_units: Vec<UnderlyingUnit> = Vec::new();
        let mut result = PackedContainer { units: result_units };

        while i < n
            invariant
                i <= n,
                n == self.units@.len(),
                n == other.units@.len(),
                i == result.units.len(),
                // CONCRETE invariant - matches concrete postcondition!
                // CRITICAL: must match what unit_combine_proof establishes
                forall|j: int| #![auto] 0 <= j < i ==>
                    forall|comp: ComponentIdx| #![auto] component_in_range(comp) ==>
                        extract_from_unit(result.units@[j], comp) ==
                        combine_values(
                            extract_from_unit(self.units@[j], comp),
                            extract_from_unit(other.units@[j], comp)
                        )
        {
            result_units = result.units;
            let u1: UnderlyingUnit = self.units[i];
            let u2: UnderlyingUnit = other.units[i];
            let combined: UnderlyingUnit = combine_units(u1, u2);

            proof {
                // Call the low-level proof
                unit_combine_proof(u1, u2, combined);
                // The proof establishes property at CONCRETE level (extract_from_unit)
                // Our invariant is also at CONCRETE level, so they connect!
            }

            result_units.push(combined);
            result = PackedContainer { units: result_units };
            i = i + 1;
        }

        result
    }
}

// ========== ABSTRACTION LEVEL GUIDE FOR PROOFS ==========
//
// **KEY PRINCIPLE:** Match postcondition and invariant abstraction level to proof level!
//
// **Use ABSTRACT level (view @) when:**
// - Proof functions reason about abstract types (Seq, Map, Set)
// - No bit-vector or low-level operations
// - Direct semantic properties
// Example: ret@[i] == combine_operation(self@[i], other@[i])
//
// **Use CONCRETE level (underlying representation access) when:**
// - Proof functions operate on underlying types (packed units, encoded data)
// - Operations with specialized proof attributes (#[verifier::...])
// - Low-level operations requiring custom extraction functions
// Example: extract_from_unit(ret.underlying@[i/N], i%N) == ...
//
// **The Connection:**
// If low_level_proof establishes:
//   extract_component(result, c) == combine(extract_component(u1, c), extract_component(u2, c))
//
// Then your postcondition MUST use extract_component too:
//   extract_component(ret.underlying@[i/N], i%N) ==
//       combine(extract_component(self.underlying@[i/N], i%N), ...)
//
// Otherwise Verus can't connect the proof to the postcondition!
//
// **For packed/low-level structures specifically:**
// - Postcondition: Use extract_component(...) at underlying level
// - Loop invariant: Use extract_component(...) at underlying level
// - Proof call: Operates on UnderlyingType
// - Result: All three at same level → verification succeeds!
//
// ============================================================

} // verus!

fn main() {}
