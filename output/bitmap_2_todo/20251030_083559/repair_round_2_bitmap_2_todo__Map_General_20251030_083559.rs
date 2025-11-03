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
        forall|loc2: u64|
            loc2 < 64 && loc2 != index ==> get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2)
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| i < 64 ==>
            get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i))
{
}


pub struct BitMap {
    bits: Vec<u64>,
}

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
            // no extra constraints needed
        ensures
            // The resulting bitmap length is 64 bits per entry in 'v'
            ret@.len() == v@.len() * 64
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as nat) < self@.len(),
        ensures
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as nat) < old(self)@.len(),
        ensures
            self@.len() == old(self)@.len(),
            self@[index as int] == bit,
            forall|i: int|
                0 <= i && i < self@.len() && i != (index as int) ==> self@[i] == old(self)@[i]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];

        // Compute bv_new depending on `bit`
        let bv_new: u64 =
            if bit {
                bv_old | (1u64 << (bit_index as u64))
            } else {
                bv_old & !(1u64 << (bit_index as u64))
            };

        // Perform the concrete update
        self.bits.set(seq_index, bv_new);

        proof {
            // Prove correctness of the updated 64-bit chunk
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);

            // Now use the macro to show the entire Seq<bool> view was updated at exactly `index`
            // (no other bits changed).
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len()
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < self@.len() ==> ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                0 <= i as int <= n as int,
                // The partially-built result covers the region [0 .. i*64) in bits
                result.bits@.len() == i as int,
                // The original 2 views have the same length in bits
                self@.len() == bm@.len(),
                // Bridging invariant: all processed bits so far match the OR condition
                forall|k: int|
                    0 <= k && k < (i as int) * 64 ==>
                    result@[k] == self@[k] || bm@[k],
            decreases n as int - i as int
        {
            res_bits = result.bits;

            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            // Perform the concrete update
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };

            proof {
                // Prove correctness of the single chunk we just produced
                bit_or_64_proof(u1, u2, or_int);

                // Show that bits [i*64 .. (i+1)*64) now match `||`
                assert forall|off: int| 0 <= off && off < 64 ==>
                    result@[(i as int) * 64 + off] == self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off]
                by {
                    // Follows from bit_or_64_proof
                }
            }

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

fn main() {}

} // verus!

// Repair Round 2 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 7
// Verified: -1, Errors: 999, Verus Errors: 7
