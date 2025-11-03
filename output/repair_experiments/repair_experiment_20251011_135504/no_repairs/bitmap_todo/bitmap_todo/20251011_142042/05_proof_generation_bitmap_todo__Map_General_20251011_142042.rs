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
    /// Returns a succinct mathematical abstraction of this bitmap:
    /// the total number of bits, and the set of bit indices that are set to true.
    ///
    /// # Formal Specification Hints
    /// This function should:
    /// - Represent the bitmap by storing its total length and the set of indices
    ///   at which the bit is true.
    /// - Thus preserve the minimal required state without storing the entire array
    ///   of booleans.
    spec fn view(&self) -> (nat, Set<nat>) {
        let total_bits = self.bits@.len() * 64;
        let set_bits = Set::new(|i: nat| i < total_bits && get_bit64!(self.bits@[(i as int)/64], ((i as int) % 64) as u64));
        (total_bits as nat, set_bits)
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
            // No additional constraints
        ensures
            ret.view().0 == v@.len() * 64,
            forall|i: nat|
                i < v@.len() * 64
                ==> ret.view().1.contains(i)
                    <==> get_bit64!(v@[(i as int)/64], ((i as int) % 64) as u64),
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
            (index as int) < self.view().0,
        ensures
            bit == get_bit64!(self.bits@[((index as int) / 64)], (((index as int) % 64)) as u64),
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
            (index as int) < old(self).view().0,
        ensures
            self.view().0 == old(self).view().0,
            forall|i: nat| i < self.view().0 ==>
                if i == (index as nat) {
                    self.view().1.contains(i) <==> bit
                } else {
                    self.view().1.contains(i) <==> old(self).view().1.contains(i)
                },
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Prove that exactly one bit changes (bit_index) and all others remain
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // Linking old(self).view() and self.view() changes only this single bit
            // (the length of bits does not change).
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
            forall|i: nat| i < self.view().0 ==>
                ret.view().1.contains(i) <==> (self.view().1.contains(i) || bm.view().1.contains(i)),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                i <= n,
                // The result bits so far are correct at indices < i
                result.bits@.len() == i,
                forall|j: int| 0 <= j < i ==> result.bits@.index(j) == self.bits@.index(j) | bm.bits@.index(j),
            decreases n - i
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

// Step 5 (proof_generation) VEval Score: Compilation Error: False, Verified: 4, Errors: 4, Verus Errors: 7
// Verified: 4, Errors: 4, Verus Errors: 7
