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


// TODO: Add bit vector proof functions here

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let ci = i / 64;
            let bi = i % 64;
            ((0x1u64 & (self.bits@[ci] >> (bi as u64))) == 1)
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            // We allow any length of v up to usize::MAX
            v@.len() <= usize::MAX
        ensures
            // The output BitMap must have the same bits-length as the input
            ret.bits@.len() == v@.len()
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            // For an immutable &self, we can use `self` directly in preconditions
            (index as nat) < 64 * self.bits@.len()
        ensures
            // The returned bit matches self@[index]
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            // For a mutable &mut self, we must refer to old(self) in requires
            (index as nat) < 64 * old(self).bits@.len()
        ensures
            // The entire view is the same as before, except the `index`th bit is updated
            self@ == old(self)@.update(index as int, bit)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = if bit {
            bv_old | 1u64 << (bit_index as u64)
        } else {
            bv_old & (!(1u64 << (bit_index as u64)))
        };
        proof {
            // Prove that set_bit64 is correct
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }
        self.bits.set(seq_index, bv_new);
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            // Both BitMaps must have the same length in their Vec<u64>
            self.bits@.len() == bm.bits@.len()
        ensures
            // ret has the same number of buckets as self
            ret.bits@.len() == self.bits@.len(),
            // Each bit in ret is OR of bits in self and bm
            forall|i: int|
                0 <= i && i < 64 * self.bits@.len()
                ==> ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            // TODO: add loop invariants
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
