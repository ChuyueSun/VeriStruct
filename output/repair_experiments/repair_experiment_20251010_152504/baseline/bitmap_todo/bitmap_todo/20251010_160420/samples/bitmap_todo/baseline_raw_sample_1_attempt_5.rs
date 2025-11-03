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

// since this wraps with `verus_proof_macro_exprs`, should use the above `get_bit64_macro` if it is going to be executable.
#[allow(unused_macros)]
macro_rules! get_bit64 {
    ($($a:tt)*) => {
        verus_proof_macro_exprs!(get_bit64_macro!($($a)*))
    }
}

/// Macro for setting a single bit in a u64 value
///
/// # Arguments
/// * `$a` - The u64 value to modify
/// * `$b` - The bit position (0-63) to set
/// * `$c` - The boolean value to set the bit to (true = 1, false = 0)
///
/// # Returns
/// A new u64 with the specified bit modified and all other bits preserved
macro_rules! set_bit64_macro {
    ($a:expr,$b:expr, $c:expr) => {{
        if $c {
            $a | 1u64 << $b
        } else {
            $a & (!(1u64 << $b))
        }
    }};
}

// since this wraps with `verus_proof_macro_exprs`, should use the above `set_bit64_macro` if it is going to be executable.
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
        let n = self.bits.len();
        Seq::new(n * 64, |i: nat|
            ((self.bits[i as int / 64] >> (i as int % 64)) & 0x1) == 1
        )
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            // No specific requirements beyond valid Vec<u64>
        ensures
            ret.bits@ == v@,
            ret.view().len() == v@.len() * 64,
            forall |i: int|
                0 <= i && i < v@.len() * 64 ==>
                    ret.view()[i] == (((v@.index(i / 64) >> (i % 64)) & 0x1) == 1),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self.bits.len() as u32 * 64,
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
            index < self.bits.len() as u32 * 64,
        ensures
            self.view()[index as int] == bit,
            forall |i: int|
                0 <= i && i < self.bits.len() as int * 64 && i != index as int
                ==> self.view()[i] == old(self).view()[i],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            /* Prove that this correctly updates exactly one bit in the view */
        }
        self.bits.set(seq_index, bv_new);
        proof {
            /* Prove the postconditions regarding self.view() here */
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),
        ensures
            ret.bits.len() == self.bits.len(),
            forall |i: int|
                0 <= i && i < ret.bits.len() as int * 64
                ==> ret.view()[i] == self.view()[i] || bm.view()[i],
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i <= n,
                result.bits.len() == i,
                forall |k: int|
                    0 <= k && k < i as int
                    ==> result.bits@[k] == self.bits@[k] | bm.bits@[k],
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                /* Prove that or_int is the correct OR of u1 and u2 */
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
