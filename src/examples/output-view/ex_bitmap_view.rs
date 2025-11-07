use vstd::prelude::*;
use vstd::seq_lib::*;

verus! {
/// Generic container of packed 64-bit chunks.
/// Shows filling in a spec fn view body for a bitmap structure.
pub struct S {
    v: Vec<u64>,
}

impl S {
// ========== INFERRED VIEW IMPLEMENTATION ==========
    /// Logical view: flatten the u64 chunks into a boolean sequence.
    /// Each u64 represents 64 bits, so total size is len * 64.
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.v@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let ci = i / 64;
            let bi = (i % 64) as u64;
            ((0x1u64 & (self.v@[ci] >> bi)) == 1)
        })
    }
// ==================================================
}
}
