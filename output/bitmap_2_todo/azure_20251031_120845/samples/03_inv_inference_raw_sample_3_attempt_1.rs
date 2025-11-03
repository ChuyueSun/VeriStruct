#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use vstd::prelude::*;
use vstd::seq_lib::*;

/*
u64 bit vector library begins
*/

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


// TODO: Add bit vector proof functions here
// You will need:
// 1. set_bit64_proof - proves that setting a bit works correctly
// 2. bit_or_64_proof - proves that OR operation on u64 works correctly

/*
u64 bit vector library ends
bitmap impl begins
*/

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    // ========== ADDED VIEW IMPLEMENTATION ==========
    closed spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let chunk = i / 64;
            let offset = i % 64;
            ((self.bits@[chunk] >> offset) & 0x1) == 1
        })
    }
    // ===============================================

    fn from(v: Vec<u64>) -> (ret: BitMap)
    // TODO: Add requires and ensures
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
    // TODO: Add requires and ensures
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
    // TODO: Add requires and ensures
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = if bit { bv_old | 1u64 << (bit_index as u64) } else { bv_old & (!(1u64 << (bit_index as u64))) };
        // TODO: add proof for set_bit64
        self.bits.set(seq_index, bv_new);
        // TODO: add proof that the sequence is correctly updated
    }

    // bitwise-OR for bitmap
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
    // TODO: Add requires and ensures
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
        // TODO: add loop invariant
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            // TODO: add proof for bit_or_64
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;

        }
        result
    }

/// Test function that verifies the correctness of BitMap operations
///
/// This function tests the basic operations of the BitMap implementation:
/// - Creating new bitmaps
/// - Setting bits
/// - Getting bits
/// - Performing OR operations between bitmaps
///
/// # Arguments
/// * `x1`, `x2`, `x3` - Test indices for bit operations
///
/// # Verification
/// The function includes formal verification requirements:
/// - All indices must be positive and less than 128
/// - Verifies that set bits can be retrieved correctly
/// - Verifies that OR operations combine bits as expected
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
 } // verus!
fn main() {}
