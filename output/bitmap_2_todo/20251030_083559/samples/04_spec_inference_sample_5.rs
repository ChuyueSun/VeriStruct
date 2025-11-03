#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use vstd::prelude::*;
use vstd::seq_lib::*;

verus! {

#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) === (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2))
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| #![auto]
            (i < 64) === (get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)))
{
}


// TODO: Add bit vector proof functions here
// You will need:
// 1. set_bit64_proof - proves that setting a bit works correctly
// 2. bit_or_64_proof - proves that OR operation on u64 works correctly


pub struct BitMap {
    bits: Vec<u64>,
}

// We keep the View trait as given:
impl View for BitMap {
    type V = Seq<bool>;

    closed spec fn view(&self) -> Self::V {
        let length = self.bits@.len() * 64;
        Seq::new(length, |i: int| {
            let chunk = i / 64;
            let offset = i % 64;
            (((self.bits@[chunk]) >> offset) & 0x1) == 1
        })
    }
}

impl BitMap {
    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            // For safety, ensure v's length is within machine limits (optional bound)
            v@.len() <= usize::MAX
        ensures
            // Physical array matches input length
            ret.bits@.len() == v@.len(),
            // View length is #bits total
            ret@.len() == v@.len() * 64,
            // Each bit in the view matches the bits from v
            forall|i: int| 0 <= i < ret@.len() ==>
                ret@[i] == ((((v@[i / 64]) >> (i % 64)) & 0x1) == 1)
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            // index must be within the total #bits in the view
            index < self@.len() as u32
        ensures
            // Return value matches the bit in the view
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            // Must be in range of old(self)'s total #bits
            index < old(self)@.len() as u32
        ensures
            // The length of the view is unchanged
            self@.len() == old(self)@.len(),
            // All bits except `index` remain the same
            forall|i: int| 0 <= i < self@.len() && i != (index as int) ==>
                self@[i] == old(self)@[i],
            // The bit at `index` is updated
            self@[index as int] == bit
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = if bit {
            bv_old | (1u64 << (bit_index as u64))
        } else {
            bv_old & !(1u64 << (bit_index as u64))
        };
        // Proof of correctness for set_bit
        proof {
            // Call set_bit64_proof if needed
        }
        self.bits.set(seq_index, bv_new);
        // Additional proof that the sequence's bits are correct
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            // Must have the same physical length to combine them
            self.bits@.len() == bm.bits@.len()
        ensures
            // The result has the same physical length
            ret.bits@.len() == self.bits@.len(),
            // The view has the same length in bits
            ret@.len() == self@.len(),
            // The result is bitwise-or of the two views
            forall|i: int| 0 <= i < ret@.len() ==>
                ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            // TODO: add loop invariant
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            // Proof of correctness for bit_or
            proof {
                // Call bit_or_64_proof if needed
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
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
}

} // verus!
fn main() {}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 7
