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
    /// Returns a sequence of boolean values representing the bitmap's contents
    /// This is a specification function used for verification purposes
    spec fn view(&self) -> Seq<bool> {
        let len_bits = self.bits.len() * 64;
        Seq::new(len_bits, |i: nat| {
            let seq_index: nat = i / 64;
            let bit_index: nat = i % 64;
            ((self.bits.index(seq_index) >> bit_index) & 0x1) == 1
        })
    }

    /// Creates a new BitMap from a vector of u64 values
    ///
    /// # Arguments
    /// * `v` - Vector of u64 values
    ///
    /// # Returns
    /// A new BitMap instance containing the provided bits
    fn from(v: Vec<u64>) -> (ret: BitMap)
    requires
        // No particular requirements on v
    ensures
        ret.view().len() == v.len() * 64,
        forall|i: nat| i < v.len() * 64 ==> ret.view().index(i) == (((v.index(i/64) >> (i%64)) & 0x1) == 1),
    {
        BitMap { bits: v }
    }

    /// Retrieves the value of a specific bit in the bitmap
    ///
    /// # Arguments
    /// * `index` - The bit position to query (0-based)
    ///
    /// # Returns
    /// * `true` if the bit is set (1)
    /// * `false` if the bit is unset (0)
    fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index < self.view().len(),
    ensures
        bit == self.view().index(index as nat),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    /// Sets or clears a specific bit in the bitmap
    ///
    /// # Arguments
    /// * `index` - The bit position to modify (0-based)
    /// * `bit` - The value to set (`true` for 1, `false` for 0)
    fn set_bit(&mut self, index: u32, bit: bool)
    requires
        index < self.view().len(),
    ensures
        self.view() == old(self).view().update(index as nat, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            // Demonstrate that only the bit at `index` changes.
            let old_v = old(self).view();
            assert_forall_by(|i: nat| {
                requires(i < self.view().len());
                if i == index as nat {
                    assert(self.view().index(i) == bit);
                } else {
                    assert(self.view().index(i) == old_v.index(i));
                }
            });
        }
        self.bits.set(seq_index, bv_new);
        proof {
            // The vector now reflects the updated bit.
            assert(self.view() == old(self).view().update(index as nat, bit));
        }
    }

    /// Performs a bitwise OR operation between two bitmaps
    ///
    /// # Arguments
    /// * `bm` - Reference to another BitMap
    ///
    /// # Returns
    /// A new BitMap containing the result of the OR operation
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self.view().len() == bm.view().len(),
    ensures
        ret.view().len() == self.view().len(),
        forall|i: nat| i < ret.view().len() ==> ret.view().index(i) == (self.view().index(i) || bm.view().index(i)),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
        invariant
            i <= n,
            result.bits.len() == i,
            forall|k: nat| k < i ==> result.bits.index(k) == self.bits.index(k) | bm.bits.index(k),
        {
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                // or_int is the bitwise OR of u1 and u2 for chunk i
            }
            result.bits.push(or_int);
            i = i + 1;
        }
        proof {
            // Now result.bits has length n and each chunk is the OR of self and bm
            assert(forall|j: nat| j < n ==> result.bits.index(j) == self.bits.index(j) | bm.bits.index(j));
        }
        result
    }
}

/// Test function that verifies the correctness of BitMap operations
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
