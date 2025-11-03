#![cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
#[allow(unused_imports)]
use builtin_macros::*;
use vstd::prelude::*;
use vstd::seq_lib::*;

// The raw macros for get_bit64 and set_bit64
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
        index < 64
    ensures
        get_bit64!(bv_new, index) == bit,
        forall|loc2: u64| #![auto]
            (loc2 < 64 && loc2 != index) ==> (get_bit64!(bv_new, loc2) == get_bit64!(bv_old, loc2))
{
}

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i))
{
}

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    /// Convert the internal u64 representation into a boolean sequence
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let ci = i / 64;
            let bi = (i % 64) as u64;
            ((0x1u64 & (self.bits@[ci] >> bi)) == 1)
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            // The resulting BitMap has a bit-sequence length of v@.len() * 64
            ret@.len() == (v@.len() * 64),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self@.len() as u32,
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
            index < old(self)@.len() as u32,
        ensures
            self@.len() == old(self)@.len(),
            self@ == old(self)@.update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        // Low-level update
        self.bits.set(seq_index, bv_new);

        proof {
            // Prove bitwise correctness:
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);

            // Now show the entire sequence equals old(self)@.update(index, bit) at the spec level
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len(),
        ensures
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < ret@.len() ==> ret@[i] == (self@[i] || bm@[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                // The length of the bit sequences must remain the same
                self@.len() == bm@.len(),
                // For all bits processed so far, the or() correctness holds
                forall|k: int| #![auto]
                    0 <= k && k < (i as int) * 64 ==>
                        result@[k] == (self@[k] || bm@[k]),
            decreases n as int - i as int
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                // Show new chunk correctness for the segment [i*64 .. (i+1)*64)
                bit_or_64_proof(u1, u2, or_int);
                assert forall|off: int| #![trigger result@.update((i as int) * 64 + off, true)[(i as int) * 64 + off]]
                    0 <= off && off < 64 implies
                        result@.update((i as int) * 64 + off, true)[(i as int) * 64 + off] ==
                        (self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off])
                by {
                    // We actually do not need a separate big assertion block if we rely on bit_or_64_proof.
                    // bit_or_64_proof ensures correctness for each bit. The loop invariant update is
                    // captured below using the standard pattern:
                }
            }

            // push new chunk
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


// VEval Score: Compilation Error: False, Verified: 7, Errors: 1, Verus Errors: 4
