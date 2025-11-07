// Example: When to use CONCRETE postconditions (packed/encoded structures)
// Shows operations where you MUST reference underlying representation

use vstd::prelude::*;

verus! {

// Proof function operates at UNDERLYING level
proof fn chunk_operation_proof(chunk1: u64, chunk2: u64, result_chunk: u64)
    requires
        result_chunk == operation_on_chunks(chunk1, chunk2)
    ensures
        // Proof establishes property about COMPONENTS within chunks
        forall|comp_idx: u64| #![auto] comp_idx < COMPONENTS_PER_CHUNK ==>
            extract_component(result_chunk, comp_idx) ==
            combine_components(
                extract_component(chunk1, comp_idx),
                extract_component(chunk2, comp_idx)
            )
{
}

pub struct PackedData {
    chunks: Vec<u64>,  // Underlying packed representation
}

impl PackedData {
    spec fn view(&self) -> Seq<ComponentType> {
        // View EXPANDS packed chunks to logical sequence
        Seq::new(self.chunks@.len() * COMPONENTS_PER_CHUNK, |i: int| {
            let chunk_idx = i / COMPONENTS_PER_CHUNK;
            let comp_idx = (i % COMPONENTS_PER_CHUNK) as u64;
            extract_component(self.chunks@[chunk_idx], comp_idx)
        })
    }

    // ========== CONCRETE POSTCONDITION (REQUIRED for packed structures) ==========
    fn read_component(&self, index: usize) -> (component: ComponentType)
        requires
            index < self@.len()
        ensures
            // CONCRETE: Use extraction at chunk level (matches view definition!)
            component == extract_component(
                self.chunks@[index / COMPONENTS_PER_CHUNK],
                (index % COMPONENTS_PER_CHUNK) as u64
            )
    {
        let chunk_idx = index / COMPONENTS_PER_CHUNK;
        let comp_idx = index % COMPONENTS_PER_CHUNK;
        extract_from_chunk(self.chunks[chunk_idx], comp_idx)
    }

    // ========== CONCRETE POSTCONDITION (REQUIRED when using chunk proofs) ==========
    fn combine(&self, other: &PackedData) -> (result: PackedData)
        requires
            self.chunks@.len() == other.chunks@.len()
        ensures
            result.chunks@.len() == self.chunks@.len(),
            // CONCRETE: Use extraction at chunk level (matches what proof establishes!)
            forall|i: int| #![auto] 0 <= i < result@.len() ==> {
                let chunk_idx = i / COMPONENTS_PER_CHUNK;
                let comp_idx = (i % COMPONENTS_PER_CHUNK) as u64;
                extract_component(result.chunks@[chunk_idx], comp_idx) ==
                combine_components(
                    extract_component(self.chunks@[chunk_idx], comp_idx),
                    extract_component(other.chunks@[chunk_idx], comp_idx)
                )
            }
    {
        let mut result_chunks: Vec<u64> = Vec::new();
        let mut i: usize = 0;

        while i < self.chunks.len()
        {
            let chunk1 = self.chunks[i];
            let chunk2 = other.chunks[i];
            let result_chunk = operation_on_chunks(chunk1, chunk2);

            proof {
                chunk_operation_proof(chunk1, chunk2, result_chunk);
                // Proof establishes properties at CHUNK level
                // Our postcondition ALSO at CHUNK level → they connect!
            }

            result_chunks.push(result_chunk);
            i = i + 1;
        }

        PackedData { chunks: result_chunks }
    }
}

// ========== WHEN TO USE CONCRETE POSTCONDITIONS ==========
//
// Use concrete (chunk-level) postconditions when:
// 1. Data is PACKED/ENCODED (multiple logical items per physical unit)
// 2. View EXPANDS underlying representation (chunks → components)
// 3. Proof functions operate on UNDERLYING type (chunks, not components)
// 4. Using specialized extraction operations
//
// KEY PATTERN:
// - If view uses: extract_component(self.chunks@[i/N], i%N)
// - Then postcondition MUST use: extract_component(ret.chunks@[i/N], i%N)
// - NOT just: ret@[i]
//
// WHY: Proof establishes properties about chunks.
// Postcondition must reference chunks to connect to proof!
//
// ==================================

} // verus!

fn main() {}
