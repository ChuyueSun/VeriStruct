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

// since this wraps with `verus_proof_macro_exprs`, should use the above `get_bit64_macro` if it is going to be executable.
#[allow(unused_macros)]
macro_rules! get_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit64_macro!($($a)*))
    }
}

/// Macro for setting a single bit in a u64 value
///
/// # Arguments
/// * `$a` - The u64 value to modify
/// * `$b` - The bit position (0-63) to set
/// * `$c` - The boolean value to set the bit to (true = 1, false = 0)
///
/// # Returns
/// A new u64 with the specified bit modified and all other bits preserved
macro_rules! set_bit64_macro {
    ($a:expr,$b:expr, $c:expr) => {{
        if $c {
            $a | 1u64 << $b
        } else {
            $a & (!(1u64 << $b))
        }
    }};
}

// since this wraps with `verus_proof_macro_exprs`, should use the above `set_bit64_macro` if it is going to be executable.
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
    /// Converts this bitmap into a specification-level sequence of booleans.
    ///
    /// Each u64 in self.bits is expanded into 64 bits. The bit at position (64 * chunk + b)
    /// is determined by get_bit64!(bits@[chunk], b).
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits@.len() * 64, |i: int| get_bit64!(self.bits@[i / 64], (i % 64) as u64))
    }

    /// Creates a new BitMap from a vector of u64 values
    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().len() == v@.len() * 64,
            forall|i: int|
                0 <= i < ret.view().len()
                ==> ret.view()[i] == get_bit64!(v@[i / 64], (i % 64) as u64),
    {
        BitMap { bits: v }
    }

    /// Retrieves the bool bit at a given index in the flattened bits representation.
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index as int < self.view().len(),
        ensures
            bit == self.view()[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    /// Sets or clears the bit at position index in the flattened bits representation.
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index as int < old(self).view().len(),
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: int| 0 <= i < self.view().len() && i != index as int
                ==> self.view()[i] == old(self).view()[i],
            self.view()[index as int] == bit,
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            // Invoke the bit-vector lemma to ensure
            // bits other than 'bit_index' are unchanged,
            // and the 'bit_index' is now set to 'bit'.
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }
        self.bits.set(seq_index, bv_new);
        proof {
            // Bridge the change from old(self).bits@[seq_index] to self.bits@[seq_index].
            // For indices i != seq_index, bits remain unchanged.
            // Hence, for positions in the view outside of (seq_index * 64)..((seq_index+1)*64),
            // the bits are the same. For the position index, the new bit is 'bit'.
        }
    }

    /// Produces a new BitMap which is the bitwise OR of self and bm.
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().len() == bm.view().len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: int|
                0 <= i < self.view().len()
                ==> ret.view()[i] == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i <= n,
                result.bits@.len() == i,
                // For all chunks that have been processed (0..i), the bits are set to the OR
                // of self.bits and bm.bits at those chunks.
                forall|chunk: int| 0 <= chunk < i ==> result.bits@[chunk] == self.bits@[chunk] | bm.bits@[chunk],
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                // Prove the resulting chunk is indeed the OR of u1 and u2.
                bit_or_64_proof(u1, u2, or_int);
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        proof {
            // Bridge from chunk-level OR to the Boolean-level view:
            // Each chunk c is the OR of the corresponding chunks from self and bm.
            // Thus, for each bit in chunk c, ret.view() is the OR result of self.view() and bm.view().
        }
        result
    }
}

fn test(x1: u32, x2: u32, x3: u32)
requires
    0 < x1 < 128,
    0 < x2 < 128,
    0 < x3 < 128,
{
    let mut bm1 = BitMap::from(vec![0u64, 0u64]);
    let mut bm2 = BitMap::from(vec![0u64, 0u64]);

    bm1.set_bit(x1, true);
    bm1.set_bit(x2, true);
    bm2.set_bit(x2, true);
    bm2.set_bit(x3, true);

    let bm1_x1 = bm1.get_bit(x1);
    let bm1_x2 = bm1.get_bit(x2);
    assert(bm1_x1 && bm1_x2);

    let bm2_x2 = bm2.get_bit(x2);
    let bm2_x3 = bm2.get_bit(x3);
    assert(bm2_x2 && bm2_x3);

    let bm3 = bm1.or(&bm2);
    let bm3_x1 = bm3.get_bit(x1);
    let bm3_x2 = bm3.get_bit(x2);
    let bm3_x3 = bm3.get_bit(x3);
    assert(bm3_x1 && bm3_x2 && bm3_x3);
}

} // verus!
fn main() {}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
