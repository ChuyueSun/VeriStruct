// Example: WHY concrete postconditions are needed (educational example)
// Demonstrates the connection between proof level and postcondition level

use vstd::prelude::*;

verus! {

// ========== THE PROOF FUNCTION (operates at CHUNK level) ==========
#[verifier::bit_vector]
proof fn operation_proof(chunk1: u64, chunk2: u64, result: u64)
    requires
        result == chunk1 | chunk2
    ensures
        // Proof establishes property at CHUNK/BIT level
        forall|bit_index: u64| #![auto] bit_index < 64 ==>
            bit_is_set(result, bit_index) ==
            (bit_is_set(chunk1, bit_index) || bit_is_set(chunk2, bit_index))
{
}

pub struct PackedBits {
    chunks: Vec<u64>,
}

impl PackedBits {
    spec fn view(&self) -> Seq<bool> {
        // View expands u64 chunks into individual bits
        Seq::new(self.chunks@.len() * 64, |i: int| {
            bit_is_set(self.chunks@[i / 64], (i % 64) as u64)
        })
    }

    // ========== DEMONSTRATION: Why abstraction level matters ==========

    // ❌ ATTEMPT 1: Abstract postcondition (UNPROVABLE!)
    /*
    fn combine_abstract(&self, other: &PackedBits) -> (result: PackedBits)
        ensures
            forall|i: int| result@[i] == (self@[i] || other@[i])
            //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            // PROBLEM: This talks about logical bits (result@[i])
            // But operation_proof talks about chunks (u64) and bit indices
            // NO CONNECTION! Verus can't prove this!
    */

    // ✅ ATTEMPT 2: Concrete postcondition (PROVABLE!)
    fn combine_concrete(&self, other: &PackedBits) -> (result: PackedBits)
        requires
            self.chunks@.len() == other.chunks@.len()
        ensures
            result.chunks@.len() == self.chunks@.len(),
            // CONCRETE: Reference chunks and bit indices directly
            forall|i: int| #![auto] 0 <= i < result@.len() ==> {
                let chunk_idx = i / 64;
                let bit_idx = (i % 64) as u64;
                bit_is_set(result.chunks@[chunk_idx], bit_idx) ==
                (bit_is_set(self.chunks@[chunk_idx], bit_idx) ||
                 bit_is_set(other.chunks@[chunk_idx], bit_idx))
            }
            // SUCCESS: This references chunks@[...] and bit indices
            // SAME as what operation_proof talks about!
            // Verus can connect them! ✓
    {
        let mut result_chunks: Vec<u64> = Vec::new();
        let mut i: usize = 0;

        while i < self.chunks.len()
        {
            let c1 = self.chunks[i];
            let c2 = other.chunks[i];
            let combined = c1 | c2;

            proof {
                operation_proof(c1, c2, combined);
                // This proves: bit_is_set(combined, bit_idx) == ...
                // Our postcondition says: bit_is_set(result.chunks@[...], bit_idx) == ...
                // MATCH! → Verification succeeds
            }

            result_chunks.push(combined);
            i = i + 1;
        }

        PackedBits { chunks: result_chunks }
    }
}

// ========== THE LESSON ==========
//
// **The Verification Chain:**
//
// 1. You call: operation_proof(chunk1, chunk2, result)
// 2. Proof establishes: bit_is_set(result, idx) == combine(bit_is_set(chunk1, idx), ...)
//    ↑ This is at CHUNK level (u64 chunks + bit indices)
//
// 3. Your postcondition says: bit_is_set(result.chunks@[i/64], i%64) == ...
//    ↑ This is ALSO at CHUNK level (chunks@ + bit indices)
//
// 4. Verus sees: "Proof talks about chunks, postcondition talks about chunks → MATCH!"
//
// 5. Result: Verification succeeds! ✓
//
// **If you use abstract:**
// 3. Your postcondition says: result@[i] == ...
//    ↑ This is at LOGICAL level (individual bits)
//
// 4. Verus sees: "Proof talks about chunks, postcondition talks about logical bits → NO MATCH!"
//
// 5. Result: Verification fails! ✗
//
// **The Rule:**
// Postcondition must use the SAME representation level as the proof function!
//
// ========================================

} // verus!

fn main() {}


+
