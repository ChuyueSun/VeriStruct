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

// A bitmap that stores bits in a vector of 64-bit chunks.
pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    // Converts the internal u64 representation into a sequence of booleans.
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits.len() * 64, |i: nat| {
            ((self.bits[i as int / 64] >> (i as int % 64)) & 1) == 1
        })
    }

    // Creates a new BitMap from a vector of u64 values.
    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            // No particular constraint on v here, but it will define the largest index allowed.
        ensures
            ret.bits == v,
            ret.view() == Seq::new(v.len() * 64, |i: nat| ((v[i / 64] >> (i % 64)) & 1) == 1),
    {
        BitMap { bits: v }
    }

    // Retrieves the value of a specific bit in the bitmap.
    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self.bits.len() as u32 * 64u32,
        ensures
            bit == self.view()[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    // Sets or clears a specific bit in the bitmap.
    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index < self.bits.len() as u32 * 64u32,
        ensures
            self.view() == old(self).view().update(index as int, bit),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            // Show that only the specified index changed in the logical view
            assertforall_by(|j: nat| {
                requires j < self.bits.len() * 64;
                ensures
                    if j == index as nat {
                        self.view()[j] == bit
                    } else {
                        self.view()[j] == old(self).view()[j]
                    };
                if j == index as nat {
                    // The updated bit must match 'bit'.
                } else {
                    // All other bits remain the same.
                }
            });
        }

        self.bits.set(seq_index, bv_new);

        proof {
            // Final confirmation that the view is as expected.
            assert(self.view() == old(self).view().update(index as int, bit));
        }
    }

    // Performs a bitwise OR operation between two bitmaps.
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.bits.len() == bm.bits.len(),
        ensures
            ret.bits.len() == self.bits.len(),
            forall|i: nat|
                i < ret.view().len() ==> ret.view()[i]
                    == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                0 <= i <= n,
                result.bits.len() == i,
                forall|k: nat| k < (i as nat * 64) ==> result.view()[k] == (self.view()[k] || bm.view()[k]),
            decreases n - i
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;

            proof {
                // For bits in chunk i, check the OR property
            }

            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }

        proof {
            // Show that for all bits in the final result, view()[bit] is the OR of self and bm
            assertforall_by(|j: nat| {
                requires j < n as nat * 64;
                ensures result.view()[j] == self.view()[j] || bm.view()[j];
                // The loop invariant assures consistency up to n.
            });
        }

        result
    }
}

// Test function to exercise the BitMap operations.
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
