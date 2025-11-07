// Example: Abstraction Level Selection for requires/ensures
// Shows when to use ABSTRACT (view @) vs CONCRETE (underlying representation) specifications

use vstd::prelude::*;

verus! {

// ========== PATTERN 1: ABSTRACT LEVEL (Standard Operations) ==========

pub struct Container<T> {
    storage: Vec<T>,
}

impl<T> Container<T> {
    // View provides logical abstraction
    spec fn view(&self) -> Seq<LogicalElement> {
        self.storage@.map(|i, item| to_logical(item))
    }

    // Use ABSTRACT postcondition for simple properties
    fn size(&self) -> (result: usize)
        ensures
            result == self@.len(),  // ABSTRACT - expresses intent clearly
    {
        self.storage.len()
    }

    // Use ABSTRACT postcondition for standard access
    fn access(&self, idx: usize) -> (element: LogicalElement)
        requires
            idx < self@.len(),
        ensures
            element == self@[idx as int],  // ABSTRACT - natural specification
    {
        to_logical(self.storage[idx])
    }

    // Use ABSTRACT postcondition for standard updates
    fn update(&mut self, idx: usize, val: LogicalElement)
        requires
            idx < old(self)@.len(),
        ensures
            self@ == old(self)@.update(idx as int, val),  // ABSTRACT - clean
    {
        self.storage.set(idx, from_logical(val));
    }
}

// ========== PATTERN 2: CONCRETE LEVEL (Low-Level Proofs) ==========

// Generic proof function that operates on underlying representation
proof fn low_level_proof(underlying1: UnderlyingType, underlying2: UnderlyingType, result: UnderlyingType)
    requires
        result == low_level_operation(underlying1, underlying2),
    ensures
        // Establishes property at CONCRETE level (about UnderlyingType)
        forall|component: ComponentIndex| in_range(component) ==>
            extract_component(result, component) ==
            combine_components(
                extract_component(underlying1, component),
                extract_component(underlying2, component)
            )
{
}

pub struct PackedStructure {
    underlying: Vec<UnderlyingType>,  // Packed/compressed representation
}

impl PackedStructure {
    spec fn view(&self) -> Seq<LogicalValue> {
        // View expands underlying packed representation to logical sequence
        Seq::new(self.underlying@.len() * ITEMS_PER_UNIT, |i: int| {
            let unit_idx = i / ITEMS_PER_UNIT;
            let component_idx = (i % ITEMS_PER_UNIT) as ComponentIndex;
            extract_component(self.underlying@[unit_idx], component_idx)
        })
    }

    // Use CONCRETE postcondition when proof operates on UnderlyingType
    fn read_component(&self, idx: usize) -> (value: LogicalValue)
        requires
            idx < self@.len(),
        ensures
            // CONCRETE - uses extract_component to match what proofs use
            value == extract_component(
                self.underlying@[idx / ITEMS_PER_UNIT],
                (idx % ITEMS_PER_UNIT) as ComponentIndex
            )
    {
        let unit_idx = idx / ITEMS_PER_UNIT;
        let comp_idx = idx % ITEMS_PER_UNIT;
        let unit = self.underlying[unit_idx];
        extract_from_unit(unit, comp_idx)
    }

    // Use CONCRETE postcondition when calling low_level_proof
    fn modify_component(&mut self, idx: usize, new_value: LogicalValue)
        requires
            idx < old(self)@.len(),
        ensures
            // CONCRETE - matches what low_level_proof establishes!
            forall|i: int| #![auto] 0 <= i < self@.len() ==> {
                let unit_i = i / ITEMS_PER_UNIT;
                let comp_i = (i % ITEMS_PER_UNIT) as ComponentIndex;
                extract_component(self.underlying@[unit_i], comp_i) ==
                if i == idx as int {
                    new_value
                } else {
                    extract_component(old(self).underlying@[unit_i], comp_i)
                }
            }
    {
        let unit_idx = idx / ITEMS_PER_UNIT;
        let comp_idx = idx % ITEMS_PER_UNIT;
        let old_unit = self.underlying[unit_idx];
        let new_unit = update_unit(old_unit, comp_idx, new_value);

        proof {
            // Proof establishes property at CONCRETE level
            modification_proof(old_unit, new_unit, comp_idx, new_value);
        }

        self.underlying.set(unit_idx, new_unit);
    }
}

// ========== ABSTRACTION LEVEL SELECTION GUIDE ==========
//
// **KEY PRINCIPLE:**
// Match the postcondition level to what proof functions can establish!
//
// **Use ABSTRACT postconditions (with @) when:**
// 1. Simple properties: length, equality, containment
// 2. Standard high-level operations on collections
// 3. No low-level proof functions involved
// 4. Direct semantic properties of the logical view
//
// Example pattern:
//   ensures ret@.len() == self@.len()
//   ensures elem == self@[index as int]
//   ensures self@ == old(self)@.update(index, value)
//
// **Use CONCRETE postconditions (underlying representation) when:**
// 1. Proof functions operate on the underlying representation type
// 2. Low-level operations: bit manipulation, packed structures, custom encodings
// 3. Using specialized proof macros or #[verifier::bit_vector]
// 4. Need to match what concrete proofs establish
//
// Example pattern:
//   ensures extract_component(ret.underlying@[i/N], i%N) ==
//           combine(extract_component(self.underlying@[i/N], i%N), ...)
//
// **Why this matters:**
// Proof functions establish properties at their operating level:
// - If proof operates on UnderlyingType → postcondition must reference UnderlyingType
// - If proof operates on LogicalView → postcondition can use @
// - Mismatch creates "abstraction gap" that Verus cannot bridge!
//
// **The Verification Chain:**
// 1. Operation: low_level_operation(underlying1, underlying2)
// 2. Proof call: low_level_proof(underlying1, underlying2, result)
// 3. Proof establishes: extract_component(result, c) == combine(extract_component(u1, c), ...)
// 4. Postcondition MUST match: extract_component(ret.underlying@[...], ...) == ...
// 5. Result: Verus can connect proof to postcondition ✓
//
// **Detection heuristic for choosing level:**
// Scan function body for:
// - Calls to proof functions with signature containing non-abstract types → CONCRETE
// - Operations on packed/encoded data (bit shifts, masks, etc.) → CONCRETE
// - Use of specialized extraction macros/functions → CONCRETE
// - Otherwise → ABSTRACT (default for clarity)
//
// ========================================================

} // verus!

fn main() {}
