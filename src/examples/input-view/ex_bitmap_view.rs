use vstd::prelude::*;
use vstd::seq_lib::*;

verus! {
    /// Generic container of packed 64-bit chunks.
    /// Demonstrates an input-view style `spec fn view` mapping packed bits
    /// into a logical `Seq<bool>` without specific identifiers/macros.
    pub struct S {
        v: Vec<u64>,
    }

    impl S {
        /// Logical view: flatten the `u64` chunks into a boolean sequence.
        spec fn view(&self) -> Seq<bool> {
            let total_bits = self.v@.len() * 64;
            Seq::new(total_bits, |i: int| {
                let ci = i / 64;
                let bi = (i % 64) as u64;
                ((0x1u64 & (self.v@[ci] >> bi)) == 1)
            })
        }
    }
}
