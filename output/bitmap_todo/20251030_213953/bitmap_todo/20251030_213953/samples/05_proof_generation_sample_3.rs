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
    /// This is a specification function used for verification purposes.
    ///
    /// The view converts the internal u64 vector into a Seq<bool> by concatenating
    /// the bits from each u64 chunk in order.
    spec fn view(&self) -> Seq<bool> {
        let total_bits: int = ( self.bits@.len() * 64 ) as int;
        Seq::new(total_bits as nat, |i: int| {
            let chunk: int = i / 64;
            let bit_index: u64 = (i % 64) as u64;
            ((0x1u64 & (self.bits@[chunk] >> bit_index)) == 1)
        })
    }

    /// Creates a new BitMap from a vector of u64 values
    ///
    /// # Arguments
    /// * `v` - Vector of u64 values where each u64 represents 64 bits
    ///
    /// # Returns
    /// A new BitMap instance containing the provided bits
    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret@ == Seq::new((v@.len() * 64) as nat, |i: int| {
                let chunk: int = i / 64;
                let bit_index: u64 = (i % 64) as u64;
                ((0x1u64 & (v@[chunk] >> bit_index)) == 1)
            })
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
            ((index as usize) / 64) < self.bits@.len(),
            (index as usize) < self.bits@.len() * 64
        ensures
            bit == self@[index as int]
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
            ((index as usize) / 64) < old(self).bits@.len(),
            (index as usize) < old(self).bits@.len() * 64
        ensures
            self.get_bit(index) == bit,
            forall|i: int| (0 <= i && i < (old(self).bits@.len() * 64) as nat && i != (index as int)) ==>
                self@[i] == old(self)@[i]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        // ========== INFERRED PROOF ==========
        proof {
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
        // ====================================
        self.bits.set(seq_index, bv_new);
        // ========== INFERRED PROOF ==========
        proof {
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
        // ====================================
    }

    /// Performs a bitwise OR operation between two bitmaps
    ///
    /// # Arguments
    /// * `bm` - Reference to another BitMap to OR with this one
    ///
    /// # Returns
    /// A new BitMap containing the result of the OR operation
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits@.len() == bm.bits@.len()
        ensures
            ret@.len() == old(self)@.len(),
            forall|i: int| (0 <= i && i < ret@.len()) ==> ret@[i] ==
                (old(self)@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        res_bits.reserve(n);
        let mut result = BitMap { bits: res_bits };
        while i < n
            // ========== INFERRED INVARIANTS ==========
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                self@.len() == bm@.len() == ((self.bits@.len() * 64) as int),
                forall|k: int| #![auto]
                    0 <= k && k < (i as int) * 64 ==>
                        view_from(result.bits@, (self.bits@.len() * 64) as int)[k]
                            == combine(self@[k], bm@[k]),
            decreases n as int - i as int
            // =========================================
        {
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let r_chunk: u64 = u1 | u2;
            // Push the new chunk
            result.bits.push(r_chunk);
            result = BitMap { bits: result.bits.clone() };
            // ========== INFERRED PROOF ==========
            proof {
                assert forall|off: int| 0 <= off && off < 64 ==>
                    view_from(result.bits@, (self.bits@.len() * 64) as int)[(i as int) * 64 + off]
                        == combine(self@[(i as int) * 64 + off], bm@[(i as int) * 64 + off])
                by {
                    chunk_op_lemma(u1, u2, r_chunk, off);
                }
            }
            // ====================================
            i = i + 1;
        }
        result
    }

    /// Spec function to get the total number of bits stored in the bitmap.
    spec fn len_bits(&self) -> int {
        (self.bits@.len() * 64) as int
    }
}

/// Spec function to convert a sequence of u64 chunks into a sequence of booleans of a given length.
spec fn view_from(chunks: Seq<u64>, len_bits: int) -> Seq<bool> {
    Seq::new(len_bits, |k: int| {
        let chunk = (k / 64) as int;
        let off   = (k % 64) as int;
        if 0 <= chunk && chunk < chunks.len() { get_bit64!(chunks[chunk], off as u64) } else { false }
    })
}

/// Spec function for combining two bits using logical OR.
spec fn combine(a: bool, b: bool) -> bool { a || b }

/// Proof of the correctness of chunk-wise bitwise OR operation at bit level.
proof fn chunk_op_lemma(a: u64, b: u64, r: u64, off: int)
    requires 0 <= off && off < 64
    ensures get_bit64!(r, off as u64) == combine(get_bit64!(a, off as u64), get_bit64!(b, off as u64))
{ }

/// Test function that verifies the correctness of BitMap operations
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
        0 < x3 < 128
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5
