use vstd::prelude::*;
fn main() {}

verus! {

#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64_macro!(bv_old, index, bit),
        index < 64,
    ensures
        get_bit64_macro!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) ==> (get_bit64_macro!(bv_new, loc2) == get_bit64_macro!(bv_old, loc2)),
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2,
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64_macro!(bv_new, i) == (get_bit64_macro!(bv1, i) || get_bit64_macro!(bv2, i)),
{
}


pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        let width = self.bits@.len() * 64;
        Seq::new(width, |i: int| get_bit64_macro!(self.bits@[i / 64], (i % 64) as u64))
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().len() == v@.len() * 64,
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as int) < self.view().len(),
        ensures
            bit == self.view()[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as int) < old(self).view().len(),
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: int| 0 <= i && i < old(self).view().len() ==>
                if i == index as int {
                    self.view()[i] == bit
                } else {
                    self.view()[i] == old(self).view()[i]
                },
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }

        self.bits.set(seq_index, bv_new);
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().len() == bm.view().len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: int| 0 <= i && i < self.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                0 <= i <= n,
                self.bits@.len() == n,
                bm.bits@.len() == n,
                result.bits@.len() == i,
                forall|k: int| 0 <= k < i ==> result.bits@[k] == self.bits@[k] | bm.bits@[k],
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
            assert(result.view().len() == self.view().len());
            assert forall |idx: int|
                0 <= idx && idx < self.view().len()
                implies result.view()[idx] == (self.view()[idx] || bm.view()[idx]) by
            {
                let chunk = idx / 64;
                let bitp = idx % 64;
                reveal(BitMap::view);
                assert(get_bit64_macro!(result.bits@[chunk], bitp as u64)
                       == get_bit64_macro!(self.bits@[chunk] | bm.bits@[chunk], bitp as u64));
                // Below is the fixed line with one less parenthesis.
                // Removed the extra parenthesis to fix the syntax error.
                assert(get_bit64_macro!(self.bits@[chunk] | bm.bits@[chunk], bitp as u64)
                       == (get_bit64_macro!(self.bits@[chunk], bitp as u64)
                           || get_bit64_macro!(bm.bits@[chunk], bitp as u64)));
            };
        }

        result
    }
}

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

// Final VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 15
// Verified: -1, Errors: 999, Verus Errors: 15
