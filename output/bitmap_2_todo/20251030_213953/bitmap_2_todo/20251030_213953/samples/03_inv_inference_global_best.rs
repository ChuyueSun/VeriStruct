#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use vstd::prelude::*;
use vstd::seq_lib::*;

verus! {

//////////////////////////
// u64 bit vector library
//////////////////////////

#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64,
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2)),
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2,
    ensures
        forall|i: u64| (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)),
{
}

//////////////////////////
// Bitmap implementation
//////////////////////////

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// View function: Logical abstraction of BitMap as a sequence of booleans.
    spec fn view(&self) -> Seq<bool> {
        let total_bits: int = (self.bits@.len() as int) * 64;
        Seq::new(total_bits, |i: int| {
            let bucket: int = i / 64;
            let bit_index: int = i % 64;
            ((0x1u64 & (self.bits@[bucket] >> (bit_index as u64))) == 1)
        })
    }

    /// Constructs a new BitMap from vector v.
    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures ret.view() == Seq::new(v@.len() * 64, |i: int| {
            let bucket: int = i / 64;
            let bit_index: int = i % 64;
            ((0x1u64 & (v@[bucket] >> (bit_index as u64))) == 1)
        })
    {
        BitMap { bits: v }
    }

    /// Returns the bit at the given index.
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            // Ensure the bucket exists.
            ((index / 64) as usize) < self.bits.len(),
        ensures
            bit == self.view()[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    /// Sets the bit at the given index to the specified boolean value.
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            ((index / 64) as usize) < self.bits.len(),
        ensures
            // The view is updated: at position index, the bit becomes 'bit',
            // and all other positions are unchanged.
            self.view() == old(self).view().update(index as int, bit)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = if bit { bv_old | (1u64 << (bit_index as u64)) }
                           else { bv_old & (!(1u64 << (bit_index as u64))) };
        // Call the proof function for setting the bit.
        set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        self.bits.set(seq_index, bv_new);
    }

    /// Returns a new BitMap that is the bitwise-OR of self and bm.
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),  // Must have the same number of buckets.
        ensures
            // For every bit position, the result is the OR of the corresponding bits.
            forall|i: int| 0 <= i < self.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                i <= n,
            invariant
                result.bits.len() == i,
            invariant
                forall |j: int| 0 <= j < i ==> result.bits[j as usize] == (self.bits[j as usize] | bm.bits[j as usize]),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            // Call the proof for ORing the bucket.
            bit_or_64_proof(u1, u2, or_int);
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
    }

    /// Test function that verifies the correctness of BitMap operations.
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
}

} // verus!

fn main() {}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
