use vstd::prelude::*;
use vstd::seq_lib::*;

/// Generic container of packed 64-bit chunks.
/// Shows an output-view style `View` implementation without relying on
/// specific identifiers from the source benchmark.
pub struct S {
    v: Vec<u64>,
}

// ========== INFERRED VIEW IMPLEMENTATION ==========
impl View for S {
    /// Logical representation as a sequence of booleans
    type V_list = Seq<bool>;

    pub closed spec fn view(&self) -> self::V_list {
        let total_bits = self.v@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let ci = i / 64;
            let bi = (i % 64) as u64;
            ((0x1u64 & (self.v@[ci] >> bi)) == 1)
        })
    }
}
// ==================================================
