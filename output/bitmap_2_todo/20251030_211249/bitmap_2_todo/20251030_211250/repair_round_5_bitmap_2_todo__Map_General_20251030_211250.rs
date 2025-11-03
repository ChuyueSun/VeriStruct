#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use vstd::prelude::*;
use vstd::seq_lib::*;

macro_rules! set_bit64 {
    ($bv:expr, $index:expr, $bit:expr) => {
        if $bit {
            $bv | (1u64 << $index)
        } else {
            $bv & !(1u64 << $index)
        }
    };
}

macro_rules! get_bit64 {
    ($bv:expr, $index:expr) => {
        (($bv >> $index) & 1u64) == 1
    };
}

verus! {

// A proof that setting a bit in a 64-bit integer works correctly.
#[verifier::bit_vector]
proof fn set_bit64_proof(bv_new: u64, bv_old: u64, index: u64, bit: bool)
    requires
        bv_new == set_bit64!(bv_old, index, bit),
        index < 64
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) === (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2))
{
}

// A proof that bitwise OR of two 64-bit integers works correctly.
#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| #![auto]
            (i < 64) === (get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i)))
{
}


// TODO: Add any additional bit vector proof functions here if needed

/*
u64 bit vector library ends
bitmap impl begins
*/

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    // -------------------------------------------------------
    //   VIEW
    // -------------------------------------------------------
    /// The logical (spec) view of the BitMap is a sequence of booleans,
    /// representing each bit in self.bits.
    pub closed spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64; // number of booleans
        Seq::new(total_bits, |i: int| {
            let ci = i / 64;
            let bi = i % 64;
            ((self.bits@[ci] >> (bi as u64)) & 0x1) == 1
        })
    }

    // -------------------------------------------------------
    //   from(v: Vec<u64>) -> BitMap
    // -------------------------------------------------------
    /// Constructs a BitMap from a vector of u64.
    /// Ensures that the returned bitmap's logical view corresponds to these 64-bit chunks.
    pub fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            // The returned bitmap's view has length = v@.len() * 64
            ret@.len() == v@.len() * 64,
            // And for each bit in that view, it properly corresponds
            // to the bits in 'v'
            forall|i: int|
                0 <= i < ret@.len()
                ==> ret@[i] == ((((v@[i / 64]) >> (i % 64)) & 0x1) == 1)
    {
        BitMap { bits: v }
    }

    // -------------------------------------------------------
    //   get_bit(&self, index: u32) -> bool
    // -------------------------------------------------------
    /// Reads a bit from the bitmap at the given index (0-based).
    /// The index must be strictly less than the total number of bits,
    /// which is self.bits@.len() * 64.
    pub fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as int) < self@.len()
        ensures
            bit === self@[(index as int)]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        ((0x1u64 & (bucket >> (bit_index as u64))) == 1)
    }

    // -------------------------------------------------------
    //   set_bit(&mut self, index: u32, bit: bool)
    // -------------------------------------------------------
    /// Sets the bit at the given index to 'bit'.
    /// The index must be strictly less than the total number of bits,
    /// which is self.bits@.len() * 64.
    /// Postcondition: the updated view matches the old view in all positions except 'index',
    /// which is updated to 'bit'.
    pub fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as int) < old(self)@.len()
        ensures
            self@ === old(self)@.update(index as int, bit)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = if bit {
            bv_old | (1u64 << (bit_index as u64))
        } else {
            bv_old & !(1u64 << (bit_index as u64))
        };
        // Proof for setting a single bit
        set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        self.bits.set(seq_index, bv_new);
        // The sequence now has the updated bit
        // Postcondition: self@ == old(self)@.update(index, bit)
    }

    // -------------------------------------------------------
    //   or(&self, bm: &BitMap) -> BitMap
    // -------------------------------------------------------
    /// Returns a new BitMap that is the bitwise-OR of 'self' and 'bm'.
    /// We require that both bitmaps have the same number of bits.
    /// Ensures that each bit in the result is the OR of corresponding bits in 'self' and 'bm'.
    pub fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len()
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i < ret@.len() === ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                // The partial result's length is i, so so far we have i 64-bit words.
                result.bits@.len() == i,
                // Each bit in the result so far is properly OR-ed
                forall|k: int|
                    0 <= k < result@.len()
                    === result@[k] == (self@[k] || bm@[k]),
                // i is bounded
                i <= n,
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            // Bitwise OR proof
            bit_or_64_proof(u1, u2, or_int);
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
            0 < x3 < 128
        ensures
            // No particular postcondition needed or derived.
            // This is a test function returning unit, so no ensures beyond correctness checks.
            true
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

fn main() {}
} // <-- Closing brace for verus! block

// Repair Round 5 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
