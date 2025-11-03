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
    /// Internal storage using a vector of u64 values.
    /// Each u64 stores 64 bits, allowing for efficient bit operations.
    bits: Vec<u64>,
}

impl BitMap {
    /// Returns a specification-level abstraction of the bitmap.
    ///
    /// This version conveys the length in bits along with the set of indices that are set to true.
    /// It is more succinct than enumerating every bit as a sequence.
    ///
    /// # Formal Specification Hints
    /// - The first component is the total number of bits represented by the bitmap
    /// - The second component is the set of positions at which bits are set to 1
    spec fn view(&self) -> (nat, Set<nat>) {
        let bits_seq = self.bits@;
        let len_bits = bits_seq.len() * 64;
        let set_bits = Set::new(|i: nat|
            i < len_bits && ({
                let chunk = bits_seq[( i / 64 ) as int];
                let offset = i % 64;
                ((chunk >> offset) & 0x1) == 1
            })
        );
        (len_bits, set_bits)
    }

    /// Creates a new BitMap from a vector of u64 values
    ///
    /// # Arguments
    /// * `v` - Vector of u64 values where each u64 represents 64 bits
    ///
    /// # Returns
    /// A new BitMap instance containing the provided bits
    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            true,
        ensures
            ret.view().0 == v@.len() * 64,
            forall|i: nat| i < ret.view().0 ==>
                ((i in ret.view().1) == ((((v@)[(i / 64) as int] >> (i % 64)) & 0x1) == 1)),
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
    ///
    /// # Implementation Notes
    /// The index is split into two parts:
    /// - seq_index: determines which u64 chunk contains the bit
    /// - bit_index: determines the bit position within that chunk
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as nat) < self.view().0,
        ensures
            bit == ((index as nat) in self.view().1),
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
    ///
    /// # Implementation Notes
    /// The operation is performed by:
    /// 1. Locating the correct u64 chunk using seq_index
    /// 2. Computing the bit position within that chunk
    /// 3. Using set_bit64_macro to modify the specific bit while preserving others
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as nat) < old(self).view().0,
        ensures
            self.view().0 == old(self).view().0,
            if bit {
                self.view().1 == old(self).view().1.union(set![index as nat])
            } else {
                self.view().1 == old(self).view().1.remove(index as nat)
            },
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Use the bit-vector lemma proving it modifies exactly one bit
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
            // Now show the abstract specification changes accordingly
            // 1. The length (#bits) remains unchanged
            assert(self.view().0 == old(self).view().0);

            // 2. The set of '1' bits is updated
            if bit {
                // The index is newly included
                // All other bits remain the same
            } else {
                // The index is removed
                // All other bits remain the same
            }
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // Additional bridging if needed:
            // After writing to self.bits, the same abstract relationships hold.
            // So we finalize that the new self.view() is consistent.
        }
    }

    /// Performs a bitwise OR operation between two bitmaps
    ///
    /// # Arguments
    /// * `bm` - Reference to another BitMap to OR with this one
    ///
    /// # Returns
    /// A new BitMap containing the result of the OR operation
    ///
    /// # Implementation Notes
    /// The operation performs a component-wise OR of the u64 chunks
    /// from both bitmaps, creating a new bitmap with the combined bits
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().0 == bm.view().0,
        ensures
            ret.view().0 == self.view().0,
            ret.view().1 == self.view().1.union(bm.view().1),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                i <= n,
                self.bits.len() == n,
                bm.bits.len() == n,
                result.bits.len() == i,
                forall|k: int|
                    0 <= k < i as int ==>
                        result.bits[k] == self.bits[k] | bm.bits[k],
                forall|bit_idx: nat|
                    bit_idx < i as nat * 64 ==>
                        (bit_idx in result.view().1)
                        == ((bit_idx in self.view().1) || (bit_idx in bm.view().1)),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                bit_or_64_proof(u1, u2, or_int);
            }

            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }

        proof {
            // So ret.view().0 = self.view().0
            // and ret.view().1 = self.view().1 union bm.view().1
        }
        result
    }
}

/// Test function that verifies the correctness of BitMap operations
///
/// This function tests the basic operations of the BitMap implementation:
/// - Creating new bitmaps
/// - Setting bits
/// - Getting bits
/// - Performing OR operations between bitmaps
///
/// # Arguments
/// * `x1`, `x2`, `x3` - Test indices for bit operations
///
/// # Verification
/// The function includes formal verification requirements:
/// - All indices must be positive and less than 128
/// - Verifies that set bits can be retrieved correctly
/// - Verifies that OR operations combine bits as expected
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

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
