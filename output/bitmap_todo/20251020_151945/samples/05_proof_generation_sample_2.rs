#[allow(unused_imports)]
use builtin_macros::*;
use vstd::prelude::*;
use vstd::seq_lib::*;

// The macros used for bitwise get/set:
macro_rules! get_bit64_macro {
    ($a:expr, $b:expr) => {{
        (0x1u64 & ($a >> $b)) == 1
    }};
}

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

/// A bitmap data structure that efficiently stores and manipulates a sequence of bits.
/// Each u64 stores 64 bits, providing block-level operations.
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// Returns the bitmap's contents as a sequence of bools (one per bit).
    spec fn view(&self) -> Seq<bool> {
        let length = (self.bits@.len() as int) * 64;
        Seq::new(length as nat, |i: int| {
            let chunk_index = i / 64;
            let bit_index = i % 64;
            ((self.bits@[chunk_index] >> bit_index) & 1) == 1
        })
    }

    /// Creates a new BitMap from a vector of u64 values.
    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            true,
        ensures
            ret@.len() == (v@.len() * 64) as nat,
            forall|i: int| 0 <= i && i < ret@.len() ==> ret@[i] == ((((v@[i / 64]) >> (i % 64)) & 1) == 1),
    {
        BitMap { bits: v }
    }

    /// Reads the bit at position `index` (0-based).
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as int) < self@.len(),
        ensures
            bit == self@[(index as int)],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    /// Sets or clears bit at position `index` (0-based).
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as int) < old(self)@.len(),
        ensures
            self@ == old(self)@.update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        // PROOF for single-bit update: use assert_seqs_equal! after the state update
        self.bits.set(seq_index, bv_new);
        proof {
            // CRITICAL #1: Must use assert_seqs_equal! macro (no manual forall).
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
    }

    /// Performs a bitwise OR operation with `bm`, returning a new Bitmap.
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len(),
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < self@.len() ==> ret@[i] == (self@[i] || bm@[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                // Basic invariants
                0 <= i as int <= n as int,
                // The result so far has i chunks:
                result.bits@.len() == i as int,
                // The views have the same length (given requirement):
                self@.len() == bm@.len(),
                // Bridge invariant: for all bits processed so far [0..i*64),
                // result's bits = self OR bm
                forall|k: int| #![auto]
                    0 <= k && k < (i as int) * 64 ==> result@[k] == (self@[k] || bm@[k]),
            decreases n as int - i as int
        {
            // Pull out the current result Vec so we can push to it:
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            // Insert the new chunk back
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };

            proof {
                // Prove that the newly added chunk covers bits [i*64 .. (i+1)*64)
                // match the property self@[k] || bm@[k].
                assert forall|off: int| 0 <= off && off < 64 ==>
                    result@[(i as int) * 64 + off]
                        == (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off])
                by {
                    bit_or_64_proof(u1, u2, or_int);
                }
            }

            i = i + 1;
        }

        result
    }
}

// Test function (unchanged)
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

fn main() {}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
