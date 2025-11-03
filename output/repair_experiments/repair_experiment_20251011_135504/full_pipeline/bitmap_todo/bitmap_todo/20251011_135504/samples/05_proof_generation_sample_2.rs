#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use builtin_macros::*;
use vstd::prelude::*;
use vstd::seq_lib::*;

macro_rules! get_bit64_macro {
    ($a:expr, $b:expr) => {{
        (0x1u64 & ($a >> $b)) == 1
    }};
}

#[allow(unused_macros)]
macro_rules! get_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit64_macro!($($a)*))
    }
}

macro_rules! set_bit64_macro {
    ($a:expr,$b:expr, $c:expr) => {{
        if $c {
            $a | 1u64 << $b
        } else {
            $a & (!(1u64 << $b))
        }
    }};
}

#[allow(unused_macros)]
macro_rules! set_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(set_bit64_macro!($($a)*))
    }
}

verus! {
#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64,
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2)),
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2,
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)),
{
}

/// # BitMap Implementation
///
/// A bitmap data structure that efficiently stores and manipulates a sequence of bits.
/// The implementation uses a vector of u64 values to store bits, where each u64
/// represents a chunk of 64 bits. This allows for efficient storage and bit operations.
///
/// The implementation is verified using the Verus verification system to ensure
/// correctness of all operations and maintain specified invariants.
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// Converts the internal u64 representation into a sequence of booleans
    /// This is used in specifications to talk about the abstract "view" of bits.
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits@.len() * 64, |i: int|
            get_bit64!(self.bits@[i / 64], (i % 64) as u64)
        )
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().len() == v@.len() * 64,
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self.view().len() as u32,
        ensures
            bit == self.view()[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index < old(self).view().len() as u32,
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: int|
                0 <= i && i < self.view().len() && i != index as int ==>
                self.view()[i] == old(self).view()[i],
            self.view()[index as int] == bit,
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Prove that set_bit64_macro sets exactly the specified bit
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // 1) The length of view is unchanged since we did not resize "bits".
            assert(self.view().len() == old(self).view().len());

            // 2) For any chunk index != seq_index, we haven't changed that u64
            assert forall|c: int|
                0 <= c < self.bits@.len() && c != seq_index as int
                implies self.bits@[c] == old(self).bits@[c];

            // 3) For the chunk == seq_index, set_bit64_proof ensures we changed
            // exactly one bit (bit_index) and preserved the others.

            // 4) Therefore, for i != index, the bit is unchanged in the abstract view
            // and for i = index, the bit is the new value 'bit'.
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().len() == bm.view().len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: int| 0 <= i && i < ret.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                // Standard loop bounds + progress
                i <= n,
                // The result so far has length i
                result.bits@.len() == i,
                // Each chunk up to i-1 is the bitwise OR
                forall|k: int| 0 <= k < i ==> result.bits@[k] == (self.bits@[k] | bm.bits@[k]),
                // Bridge invariants at the view-level
                result.view().len() == i * 64,
                forall|bit_idx: int|
                    0 <= bit_idx < i * 64 ==>
                    result.view()[bit_idx] == (self.view()[bit_idx] || bm.view()[bit_idx]),
            decreases n - i
        {
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                // Prove the bitwise OR property for just these 64 bits
                bit_or_64_proof(u1, u2, or_int);
            }

            // Push the chunk
            let mut temp = result.bits;
            temp.push(or_int);
            result = BitMap { bits: temp };

            i = i + 1;
        }

        // At loop end, i == n. So result.bits@.len() == n
        // => result.view().len() == n*64 == self.view().len()
        // And each chunk is the bitwise OR of self.bits@ and bm.bits@.

        result
    }
}

fn main() {}


// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
