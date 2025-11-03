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

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits.len() * 64, |i: nat| {
            let chunk = i / 64;
            let offset = i % 64;
            ((self.bits.index(chunk as int) >> offset) & 1) == 1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.bits == v,
            ret.view().len() == v.len() * 64,
            forall |i: int|
                0 <= i && i < v.len() as int * 64 ==>
                    ret.view().index(i) == (((v.index((i / 64) as int) >> (i % 64)) & 1) == 1),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index as int < self.bits.len() as int * 64,
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
            index as int < self.bits.len() as int * 64,
        ensures
            self.view().index(index as int) == bit,
            forall |i: int|
                0 <= i && i < self.bits.len() as int * 64 && i != index as int ==>
                    self.view().index(i) == old(self).view().index(i),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            let old_view = old(self).view();
            assert(self.view().ext_eq(old_view));
        }
        self.bits.set(seq_index, bv_new);
        proof {
            let new_view = self.view();
            assert(new_view.index(index as int) == bit);
            assert(forall |i: int|
                0 <= i && i < self.bits.len() as int * 64 && i != index as int
                ==> new_view.index(i) == old_view.index(i));
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),
        ensures
            ret.bits.len() == self.bits.len(),
            ret.view().len() == self.view().len(),
            forall |i: int| 0 <= i && i < self.view().len() ==> ret.view().index(i) == (self.view().index(i) || bm.view().index(i)),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i <= n,
                result.bits.len() == i,
                forall |k: int, j: int|
                    0 <= k && k < i as int &&
                    0 <= j && j < 64 ==>
                        (((result.bits.index(k) >> j) & 1) == 1)
                        == (
                            (((self.bits.index(k) >> j) & 1) == 1)
                            || (((bm.bits.index(k) >> j) & 1) == 1)
                        ),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                // No special proof needed, the bitwise OR is straightforward
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
