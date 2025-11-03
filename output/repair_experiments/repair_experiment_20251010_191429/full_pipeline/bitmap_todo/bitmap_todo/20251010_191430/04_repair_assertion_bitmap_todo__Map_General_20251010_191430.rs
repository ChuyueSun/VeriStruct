use vstd::prelude::*;
fn main() {}

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

macro_rules! set_bit64_macro {
    ($a:expr,$b:expr, $c:expr) => {{
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

/// A bitmap data structure that efficiently stores and manipulates a sequence of bits,
/// using a vector of u64 values to store chunks of 64 bits each.
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits@.len() * 64, |i: int|
            get_bit64!(self.bits@[i / 64], (i % 64) as u64)
        )
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().len() == v@.len() * 64,
            forall|i: int| 0 <= i && i < ret.view().len()
                ==> ret.view()[i] == get_bit64!(v@[i / 64], (i % 64) as u64),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self.bits@.len() as u32 * 64,
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
            index < old(self).bits@.len() as u32 * 64,
        ensures
            self.view() == old(self).view().update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }
        self.bits.set(seq_index, bv_new);
        proof {
            assert forall|k: int|
                0 <= k < old(self).view().len()
                implies
                if k == index as int {
                    self.view()[k] == bit
                } else {
                    self.view()[k] == old(self).view()[k]
                }
            by { };
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits@.len() == bm.bits@.len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: int| 0 <= i && i < ret.view().len()
                ==> ret.view()[i] == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                i <= n,
                self.bits@.len() == n,
                bm.bits@.len() == n,
                result.bits.len() == i,
                forall|j: int| 0 <= j < i as int ==> result.bits@[j] == self.bits@[j] | bm.bits@[j],
                forall|bitidx: int|
                    0 <= bitidx < 64 * (i as int)
                    ==> result.view()[bitidx] == self.view()[bitidx] || bm.view()[bitidx],
        {
            res_bits = result.bits;
            proof {
                assert(self.bits@.len() == n);
                assert(bm.bits@.len() == n);
                assert(i < self.bits@.len());
                assert(i < bm.bits@.len());
            }
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
