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
    spec fn view(&self) -> Seq<bool> {
        let total_bits = self.bits@.len() * 64;
        Seq::new(total_bits, |i: int| {
            let chunk = i / 64;
            let offset = i % 64;
            ((self.bits@[chunk] >> offset) & 0x1u64) == 1
        })
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        requires
            v@.len() * 64 <= usize::MAX as nat
        ensures
            ret@.len() == v@.len() * 64
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as nat) < self@.len()
        ensures
            bit == self@[index as int]
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        let retb: bool = get_bit64_macro!(bucket, bit_index as u64);
        proof {
            assert(self.bits@[seq_index as int] == bucket);
            assert(self@[index as int] == (((bucket >> (bit_index as int)) & 0x1u64) == 1));
            assert(retb == ((((bucket >> (bit_index as int)) & 0x1u64) == 1)));
            assert(retb == self@[index as int]);
        }
        retb
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as nat) < old(self)@.len()
        ensures
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
            ret@.len() == self@.len(),
            forall|i: int| 0 <= i && i < ret@.len() ==>
                ret@[i] == (self@[i] || bm@[i])
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                self@.len() == bm@.len(),
                forall|k: int| #![auto]
                    0 <= k && k < (i as int) * 64 ==>
                    result@[k] == (self@[k] || bm@[k]),
            decreases n as int - i as int
        {
            proof {
                assert(i < n);
                assert(n == self.bits.len());
                assert(n == bm.bits.len());
                assert(i < self.bits.len());
                assert(i < bm.bits.len());
            }
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                bit_or_64_proof(u1, u2, or_int);
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
            proof {
                assert(
                    forall|k: int|
                        0 <= k && k < (i as int) * 64 ==>
                        result@[k] == (self@[k] || bm@[k])
                );
            }
        }
        proof {
            assert(result@.len() == (n as int) * 64);
            assert((n as int)*64 == self@.len());
            assert(result@.len() == self@.len());
            assert(
                forall|j: int| 0 <= j && j < result@.len() ==>
                    result@[j] == (self@[j] || bm@[j])
            );
        }
        result
    }
}

fn test(x1: u32, x2: u32, x3: u32)
requires
    0 < x1 < 128,
    0 < x2 < 128,
    0 < x3 < 128
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

// VEval Score: Compilation Error: False, Verified: 5, Errors: 3, Verus Errors: 4
