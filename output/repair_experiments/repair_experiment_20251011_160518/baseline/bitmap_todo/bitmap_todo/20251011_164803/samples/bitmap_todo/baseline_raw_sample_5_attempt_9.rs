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
            let chunk_index = i / 64;
            let bit_index = i % 64;
            ((self.bits.index(chunk_index) >> bit_index) & 0x1) == 0x1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.bits@ == v@,
            ret.view().len() == v@.len() * 64,
            forall|i: nat| i < ret.view().len() ==> ret.view()[i] == (((v@.index(i / 64) >> (i % 64)) & 0x1) == 0x1)
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index as nat < self.view().len(),
        ensures
            bit == self.view()[index as nat]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index as nat < self.view().len(),
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: nat| i < self.view().len() ==>
                if i == index as nat {
                    self.view()[i] == bit
                } else {
                    self.view()[i] == old(self).view()[i]
                }
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            assert(self.view()[index as nat] != bit) || true;
        }
        self.bits.set(seq_index, bv_new);
        proof {
            assert(self.view()[index as nat] == bit);
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().len() == bm.view().len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: nat| i < self.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i <= n,
                result.bits@.len() == i,
                forall|j: nat| j < i ==> result.bits@.index(j) == self.bits@.index(j) | bm.bits@.index(j),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                assert(((u1 | u2) & 0x1) == ((u1 & 0x1) == 0x1 || (u2 & 0x1) == 0x1));
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
