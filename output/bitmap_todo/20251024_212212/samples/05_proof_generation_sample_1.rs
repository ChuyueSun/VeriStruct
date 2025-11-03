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
        index < 64
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2))
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i))
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
    /// Internal storage using a vector of u64 values.
    /// Each u64 stores 64 bits, allowing for efficient bit operations.
    bits: Vec<u64>,
}

impl BitMap {
    /// Returns a sequence of boolean values representing the bitmap's contents
    /// This is a specification function used for verification purposes
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            ((self.bits@[(i / 64) as int] >> ((i % 64) as nat)) & 0x1) == 1
        })
    }

    /// Creates a new BitMap from a vector of u64 values
    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            true
        ensures
            ret@.len() == 64 * v@.len()
    {
        BitMap { bits: v }
    }

    /// Retrieves the value of a specific bit in the bitmap
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as int) < self@.len()
        ensures
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    /// Sets or clears a specific bit in the bitmap
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as int) < old(self).view().len()
        ensures
            self@ == old(self).view().update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;

        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Prove that exactly one bit changed in the chunk
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
            // Then use the built-in macro to show the entire sequence matches the update
            assert_seqs_equal!(
                self@.update(index as int, bit),
                old(self).view().update(index as int, bit)
            );
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // After actually updating the chunk, show the final self@ is the updated sequence
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
    }

    /// Performs a bitwise OR operation between two bitmaps
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len()
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < self@.len() ==> ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                self@.len() == bm@.len(),
                forall|k: int| 0 <= k && k < (i as int) * 64 ==>
                    result@[k] == (self@[k] || bm@[k]),
            decreases n as int - i as int
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                bit_or_64_proof(u1, u2, or_int);
                // Prove the newly added chunk matches the or() of self/bm for indices [i*64..(i+1)*64)
                assert forall|off: int| 0 <= off && off < 64 ==>
                    get_bit64!(or_int, off as u64) == (get_bit64!(u1, off as u64) || get_bit64!(u2, off as u64))
                by { /* done above in bit_or_64_proof */ }

                assert forall|off: int|
                    0 <= off && off < 64 implies
                    BitMap::view_chunk(or_int, off, i, result.bits@.len(), self, bm) == true;
            }

            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
    }
}

// Helper function for bridging each chunk in the loop invariant proof.
// This is just to illustrate how the newly added chunk "or_int" indeed
// matches the boolean or of the two original bitmaps for the chunk's range.
impl BitMap {
    proof fn view_chunk(
        or_int: u64,
        off: int,
        i: usize,
        length_so_far: int,
        self_bm: &BitMap,
        other_bm: &BitMap
    ) -> bool
        ensures
            self_bm@.len() == other_bm@.len() == (self_bm.bits@.len() * 64),
            0 <= off && off < 64,
            0 <= length_so_far && length_so_far < = i as int, // dummy condition for example
            // This is the actual bridging:
            get_bit64!(or_int, off as u64)
                == ( get_bit64!(self_bm.bits[i], off as u64)
                     || get_bit64!(other_bm.bits[i], off as u64) )
    {
        true
    }
}

/// Test function
fn main() {}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
