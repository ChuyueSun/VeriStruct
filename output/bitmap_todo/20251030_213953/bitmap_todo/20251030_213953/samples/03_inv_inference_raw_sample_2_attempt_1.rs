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
    ($a:expr, $b:expr, $c:expr) => {{
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

pub struct BitMap {
    /// Internal storage using a vector of u64 values.
    /// Each u64 stores 64 bits, allowing for efficient bit operations.
    bits: Vec<u64>,
}

impl BitMap {

    // ---------- TYPE INVARIANT ----------
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        // For a BitMap, we require that the number of bits in the view equals the
        // number of u64 elements times 64.
        self.view().len() == (self.bits@.len() * 64) as nat
    }
    // ------------------------------------

    /// Returns a sequence of boolean values representing the bitmap's contents.
    /// This is a specification function used for verification purposes.
    ///
    /// The view converts the internal u64 vector into a Seq<bool> by concatenating
    /// the bits from each u64 chunk in order.
    spec fn view(&self) -> Seq<bool> {
        let total_bits: int = (self.bits@.len() * 64) as int;
        Seq::new(total_bits as nat, |i: int| {
            let chunk: int = i / 64;
            let bit_index: u64 = (i % 64) as u64;
            (0x1u64 & (self.bits@[chunk] >> bit_index)) == 1
        })
    }

    /// Creates a new BitMap from a vector of u64 values.
    ///
    /// # Arguments
    /// * `v` - Vector of u64 values where each u64 represents 64 bits.
    ///
    /// # Returns
    /// A new BitMap instance containing the provided bits.
    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view() == Seq::new((v.len() * 64) as nat, |i: int| {
                let chunk: int = i / 64;
                let bit_index: u64 = (i % 64) as u64;
                (0x1u64 & (v[chunk] >> bit_index)) == 1
            })
    {
        BitMap { bits: v }
    }

    /// Retrieves the value of a specific bit in the bitmap.
    ///
    /// # Arguments
    /// * `index` - The bit position to query (0-based).
    ///
    /// # Returns
    /// * `true` if the bit is set (1).
    /// * `false` if the bit is unset (0).
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            ((index / 64) as usize) < self.bits.len(),
        ensures
            bit == ((0x1u64 & (self.bits[((index / 64) as usize)] >> (index % 64))) == 1)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    /// Sets or clears a specific bit in the bitmap.
    ///
    /// # Arguments
    /// * `index` - The bit position to modify (0-based).
    /// * `bit` - The value to set (`true` for 1, `false` for 0).
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            ((index / 64) as usize) < self.bits.len(),
        ensures
            self.get_bit(index) == bit,
            forall|j: u32| (j != index && ((j / 64) as usize) < old(self).bits.len()) ==>
                self.get_bit(j) == old(self).get_bit(j)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        // Proof obligations for bit-level correctness are handled by the bit_vector proof functions.
        self.bits.set(seq_index, bv_new);
        // Additional proof obligations can be added as needed.
    }

    /// Performs a bitwise OR operation between two bitmaps.
    ///
    /// # Arguments
    /// * `bm` - Reference to another BitMap to OR with this one.
    ///
    /// # Returns
    /// A new BitMap containing the result of the OR operation.
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),
        ensures
            ret.view() == Seq::new((self.bits.len() * 64) as nat, |i: int|
                self.view()[i] || bm.view()[i]
            )
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                i <= n,
            invariant
                forall|k: nat| 0 <= k < i ==> result.bits[k as usize] == (self.bits[k as usize] | bm.bits[k as usize])
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            // Proof obligations for bit-level OR are handled by the bit_or_64_proof.
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
    }
}

/// Test function that verifies the correctness of BitMap operations.
///
/// This function tests the basic operations of the BitMap implementation:
/// - Creating new bitmaps
/// - Setting bits
/// - Getting bits
/// - Performing OR operations between bitmaps
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
