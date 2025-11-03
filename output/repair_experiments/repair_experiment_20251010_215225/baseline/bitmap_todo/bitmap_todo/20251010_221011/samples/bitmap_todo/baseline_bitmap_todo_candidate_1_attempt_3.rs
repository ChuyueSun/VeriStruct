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

verus! {

spec fn bool_seq_of_u64(x: u64) -> Seq<bool> {
    Seq::new(64, |i: nat| ((x >> i) & 0x1u64) == 1)
}

spec fn bits_to_view(v: Seq<u64>) -> Seq<bool> {
    Seq::flatten(v.map(|x| bool_seq_of_u64(x)))
}

/// # BitMap Implementation
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// Convert internal representation into a sequence of booleans
    spec fn view(&self) -> Seq<bool> {
        bits_to_view(self.bits@)
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
    requires
        // no specific preconditions
    ensures
        ret.bits@ == v@,
        ret.view() == bits_to_view(v@),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        index as nat < self.bits@.len() * 64,
    ensures
        bit == self.view().index(index as int),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
    requires
        index as nat < self.bits@.len() * 64,
    ensures
        self.view() == old(self).view().update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            assert(self.view().update(index as int, bit).ext_eq(
                bits_to_view(self.bits@).update(index as int, bit));
        }

        self.bits.set(seq_index, bv_new);

        proof {
            assert(self.view().ext_eq(bits_to_view(self.bits@)));
            assert(self.view() == old(self).view().update(index as int, bit));
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self.bits@.len() == bm.bits@.len(),
    ensures
        ret.bits@.len() == self.bits@.len(),
        forall|i: nat| i < ret.bits@.len() ==> ret.bits@[i] == self.bits@[i] | bm.bits@[i],
        ret.view() == Seq::new(self.view().len(), |i| self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
        invariant
            0 <= i <= n,
            result.bits@.len() == i,
            forall|j: nat| j < i ==> result.bits@[j] == self.bits@[j] | bm.bits@[j],
        decreases n - i
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                assert(result.bits@.len() == i);
            }

            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
    }
}

/// Test function that verifies correctness of BitMap operations
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
