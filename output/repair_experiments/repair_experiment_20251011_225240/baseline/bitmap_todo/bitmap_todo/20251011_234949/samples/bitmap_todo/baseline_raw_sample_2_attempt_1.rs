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

spec fn get_bit_spec(a: u64, index: nat) -> bool {
    ((a >> index) & 0x1) == 1
}

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        let len = self.bits.len();
        let total_bits = len * 64;
        Seq::new_total(total_bits, |i: nat| {
            let seq_index = i / 64;
            let bit_index = i % 64;
            get_bit_spec(self.bits.index(seq_index), bit_index)
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
    requires
        true
    ensures
        ret.view().len() == v.len() * 64,
        forall|i: nat| i < ret.view().len() ==>
            ret.view().index(i)
            == get_bit_spec(v.index(i / 64), i % 64)
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
    requires
        (index as nat) < self.bits.len() as nat * 64
    ensures
        bit == self.view().index(index as nat)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
    requires
        (index as nat) < self.bits.len() as nat * 64
    ensures
        self.view().len() == old(self).view().len(),
        self.view() == old(self).view().update(index as nat, bit)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            let old_view = old(self).view();
            let updated_view = old_view.update(index as nat, bit);
            assert(updated_view.len() == old_view.len());
        }
        self.bits.set(seq_index, bv_new);
        proof {
            assert(self.view().len() == old(self).view().len());
            assert(forall |i: nat| i < self.view().len() ==>
                self.view().index(i)
                == if i == index as nat {
                    bit
                } else {
                    old(self).view().index(i)
                }
            );
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self.bits.len() == bm.bits.len()
    ensures
        ret.view().len() == self.view().len(),
        forall|i: nat| i < self.view().len() ==>
            ret.view().index(i)
            == (self.view().index(i) || bm.view().index(i))
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
        invariant
            i <= n,
            result.bits.len() == i,
            forall|k: nat| k < i ==> result.bits.index(k) == (self.bits.index(k) | bm.bits.index(k))
        decreases n - i
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                assert(get_bit_spec(u1, 0) || get_bit_spec(u2, 0) == get_bit_spec(or_int, 0));
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        proof {
            assert(result.view().len() == self.view().len());
            assert(forall |idx: nat| idx < self.view().len() ==>
                result.view().index(idx)
                == (self.view().index(idx) || bm.view().index(idx)));
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
