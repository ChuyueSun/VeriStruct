use builtin_macros::*;
use vstd::prelude::*;
use vstd::assert_seqs_equal;
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

/// Macro for setting a single bit in a u64 value
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
{ }

#[verifier::bit_vector]
proof fn bit_or_64_proof(bv1: u64, bv2: u64, bv_new: u64)
    requires
        bv_new == bv1 | bv2
    ensures
        forall|i: u64| #![auto]
            (i < 64) ==> get_bit64!(bv_new, i) == (get_bit64!(bv1, i) || get_bit64!(bv2, i))
{ }

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let chunk = i / 64;
            let bitpos = i % 64;
            ((self.bits@[chunk] >> bitpos) & 0x1) == 1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret@ == Seq::new((v@.len() * 64) as nat, |i: int|
                (((v@[i / 64]) >> (i % 64)) & 0x1) == 1)
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self@.len()
        ensures
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index < old(self)@.len()
        ensures
            self.bits@.len() == old(self).bits@.len(),
            self@ == old(self)@.update(index as int, bit)
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }
        self.bits.set(seq_index, bv_new);
        proof {
            assert_seqs_equal!(
                self@,
                old(self).view().update(index as int, bit)
            );
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len()
        ensures
            ret@ == Seq::new(self@.len(), |i: int| self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64>= Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                self.bits@.len() == bm.bits@.len() == n as int,
                self@.len() == bm@.len(),
                forall|k: int|
                    0 <= k && k < result.bits@.len() * 64 ==>
                    result@[k] == self@[k] || bm@[k],
            decreases n as int - i as int
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                bit_or_64_proof(u1, u2, or_int);
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            proof {
                assert(forall|off: int| 0 <= off && off < 64 ==> result@[(i as int) * 64 + off] == self@[(i as int) * 64 + off] || bm@[(i as int) * 64 + off]);
            }
            i = i + 1;
            proof {
                // Reassert the invariant after increment
                assert(forall|k: int|
                    0 <= k && k < result.bits@.len() * 64 ==>
                    result@[k] == self@[k] || bm@[k]);
            }
        }
        result
    }
}

fn main() { }


// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
