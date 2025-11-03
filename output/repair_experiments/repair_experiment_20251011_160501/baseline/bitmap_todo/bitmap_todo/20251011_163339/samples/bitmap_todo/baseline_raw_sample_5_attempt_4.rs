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
        Seq::new(self.bits.len() * 64, |idx: nat| {
            let chunk_index = idx / 64;
            let bit_index = idx % 64;
            ((self.bits[chunk_index] >> bit_index) & 1) == 1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            // No specific requirement here except that v can be any vector
        ensures
            ret.bits == v,
            ret.view() == Seq::new(v@.len() * 64, |idx: nat| {
                let chunk = idx / 64;
                let bit_idx = idx % 64;
                ((v@[chunk]) >> bit_idx) & 1 == 1
            }),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index as int < self.view().len(),
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
            index as int < old(self).view().len(),
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: int|
                0 <= i < old(self).view().len() ==> (
                    if i == index as int {
                        self.view().index(i) == bit
                    } else {
                        self.view().index(i) == old(self).view().index(i)
                    }
                ),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        // Proof that only the indexed bit changed:
        proof {
            assert_forall_by(|k: int| {
                requires(k >= 0 && k < self.view().len());
                if k == index as int {
                    assert(((bv_new >> (bit_index as int)) & 1) == (if bit { 1 } else { 0 }));
                } else {
                    assert((((bv_new >> (k % 64)) & 1) == ((bv_old >> (k % 64)) & 1)));
                }
            });
        }
        self.bits.set(seq_index, bv_new);
        // Proof that the final view matches the expected updated view:
        proof {
            assert(self.view().len() == old(self).view().len());
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),
        ensures
            ret.bits.len() == self.bits.len(),
            ret.view().len() == self.view().len(),
            forall|i: int|
                0 <= i < self.view().len() ==>
                    ret.view().index(i) == (self.view().index(i) || bm.view().index(i)),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                i <= n,
                result.bits.len() == i,
                forall|k: int| 0 <= k < i * 64 ==> result.view().index(k) == (self.view().index(k) || bm.view().index(k)),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                // For all bits j in [0..64), the j-th bit of or_int is the OR of u1 and u2's j-th bits.
                assert_forall_by(|j: int| {
                    requires(j >= 0 && j < 64);
                    let b1 = ((u1 >> j) & 1) == 1;
                    let b2 = ((u2 >> j) & 1) == 1;
                    let r = ((or_int >> j) & 1) == 1;
                    assert(r == (b1 || b2));
                });
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
