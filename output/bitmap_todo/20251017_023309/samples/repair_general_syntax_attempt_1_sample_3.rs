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

/// # BitMap Implementation
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// Specification-level view function returning the bitmap as a sequence of bools.
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |idx: int| {
            let chunk = idx / 64;
            let bit_idx = idx % 64;
            let w = self.bits@[chunk];
            ((w >> (bit_idx as nat)) & 0x1) == 0x1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret@.len() == (v@.len() as int) * 64,
            forall|i: int|
                0 <= i && i < (v@.len() as int)*64 ==>
                    ret@[i] == (((v@[(i / 64) as int] >> (i % 64)) & 0x1) == 0x1),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as int) < self@.len(),
        ensures
            bit == self@[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as int) < old(self)@.len(),
        ensures
            self@.len() == old(self)@.len(),
            forall|i: int| 0 <= i && i < old(self)@.len() ==>
                if i == index as int {
                    self@[i] == bit
                } else {
                    self@[i] == old(self)@[i]
                },
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Prove that only this bit changed
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // 1) The length of the bitmap is unchanged
            assert(self@.len() == old(self).view().len());

            // 2) For all bits i, check if i == index => updated
            //    else => unchanged
            assert forall|i_int: int| 0 <= i_int < old(self)@.len() implies {
                if i_int == index as int {
                    self@[i_int] == bit
                } else {
                    self@[i_int] == old(self)@[i_int]
                }
            } by {
                let chunk_i = i_int / 64;
                let bit_i = i_int % 64;
                if chunk_i as usize == seq_index && bit_i as u32 == bit_index {
                    // This is exactly the changed bit
                } else {
                    // Different bit => remains same
                }
            };
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len(),
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < self@.len() ==>
                ret@[i] == (self@[i] || bm@[i]),
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
                // So far, we've processed i chunks
                result.bits@.len() == i,
                // For each chunk j < i, the bits in result correspond to the OR of self and bm
                forall|j: int| 0 <= j < i ==> {
                    let or_chunk = result.bits@[j];
                    let chunk1 = self.bits@[j];
                    let chunk2 = bm.bits@[j];
                    // Each bit in or_chunk is the OR of chunk1, chunk2
                    forall|bitidx: u64|
                        bitidx < 64 ==>
                            #[trigger] get_bit64!(or_chunk, bitidx)
                            == (#[trigger] get_bit64!(chunk1, bitidx)
                                || #[trigger] get_bit64!(chunk2, bitidx))
                }
            decreases n - i
        {
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                // Prove bitwise OR property
                bit_or_64_proof(u1, u2, or_int);
            }

            // push to result
            res_bits = result.bits;
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };

            i = i + 1;
        }
        // After the loop, result has n chunks => ret has same # of chunks
        // => total bits = n*64 => matches self@.len().
        // And each bit is the OR of self/bm bits

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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
