fn main() {}

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
proof fn lemma_get_bit64_equiv(a: u64, b: u64)
    ensures
        get_bit64_macro!(a, b) == (((a >> b) & 1u64) == 1u64),
{
}

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

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> (nat, Set<nat>) {
        let l = self.bits@.len() * 64;
        let s = Set::new(|i: nat| i < l && ((((self.bits@[( i / 64 ) as int]) >> (i % 64)) & 1) == 1));
        (l, s)
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret@.0 == v@.len() * 64,
            ret@.1 == Set::new(|i: nat|
                i < (v@.len() * 64)
                && ((((v@[(i / 64) as int]) >> (i % 64)) & 1) == 1)
            ),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as nat) < self@.0,
        ensures
            bit == self@.1.contains(index as nat),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        let result = get_bit64_macro!(bucket, bit_index as u64);

        proof {
            lemma_get_bit64_equiv(bucket, bit_index as u64);
            let ghost bucket_spec = self.bits@[seq_index as int];
            assert(bucket == bucket_spec);

            assert(
                result
                == (
                    ((bucket_spec >> bit_index) & 1u64) == 1u64
                )
            );
            assert(
                (
                    ((self.bits@[(index / 64) as int] >> (index % 64)) & 1u64) == 1u64
                ) <==> self@.1.contains(index as nat)
            );
            assert(result == self@.1.contains(index as nat));
        }
        result
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as nat) < old(self)@.0,
        ensures
            self@.0 == old(self)@.0,
            if bit {
                self@.1 == old(self)@.1.insert(index as nat)
            } else {
                self@.1 == old(self)@.1.remove(index as nat)
            },
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
            let (old_len, old_set) = old(self)@;
            let (new_len, new_set) = self@;
            assert(new_len == old_len);

            assert forall |i: nat| i < old_len implies
                new_set.contains(i)
                == ( if i == index as nat { bit } else { old_set.contains(i) } )
            by {
                let seg_i = i / 64;
                let off_i = i % 64;
                if seg_i as usize == seq_index && off_i as u32 == bit_index {
                } else {
                }
            };

            if bit {
                assert(new_set == old_set.insert(index as nat));
            } else {
                assert(new_set == old_set.remove(index as nat));
            }
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.0 == bm@.0,
        ensures
            ret@.0 == self@.0,
            ret@.1 == self@.1.union(bm@.1),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                i <= n,
                result.bits@.len() == i,
                forall|k: int| 0 <= k < i ==> result.bits@.index(k) == (self.bits@.index(k) | bm.bits@.index(k)),
            decreases n - i
        {
            res_bits = result.bits;
            proof {
                assert(n == self.bits@.len());
                assert(i < self.bits@.len());
            }
            let u1: u64 = self.bits[i];

            proof {
                assert(self.bits@.len() == bm.bits@.len());
                assert(i < bm.bits@.len());
            }
            let u2: u64 = bm.bits[i];

            let or_int: u64 = u1 | u2;
            proof {
                bit_or_64_proof(u1, u2, or_int);
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        proof {
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

}
