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
    spec fn view(&self) -> Seq<bool> {
        Seq::new(self.bits@.len() * 64, |i: int| get_bit64!(self.bits@[i / 64], (i % 64) as u64))
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().len() == v@.len() * 64,
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self.view().len() as u32,
        ensures
            bit == self.view()[index as int],
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index < old(self).view().len() as u32,
        ensures
            self.view().len() == old(self).view().len(),
            forall|i: int|
                0 <= i && i < self.view().len() && i != index as int ==>
                self.view()[i] == old(self).view()[i],
            self.view()[index as int] == bit,
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);

        proof {
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
            assert(self.bits@.len() == old(self).bits@.len());
        }

        self.bits.set(seq_index, bv_new);

        proof {
            assert(self.view().len() == old(self).view().len());
            assert_forall_by(|ii: int| {
                requires(0 <= ii && ii < self.view().len() && ii != index as int);
                ensures(self.view()[ii] == old(self).view()[ii]);
                {
                    let chunk_before = (ii / 64) as int;
                    let chunk_index = (index as int) / 64;
                    if chunk_before != chunk_index {
                        assert(self.bits@[chunk_before] == old(self).bits@[chunk_before]);
                    } else {
                        let bit_before = (ii % 64) as u64;
                        assert(get_bit64!(self.bits@[chunk_before], bit_before)
                               == get_bit64!(old(self).bits@[chunk_before], bit_before));
                    }
                }
            });
            assert(self.view()[index as int] == bit);
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().len() == bm.view().len(),
        ensures
            ret.view().len() == self.view().len(),
            forall|i: int| 0 <= i && i < ret.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };

        while i < n
            invariant
                i <= n,
                n == self.bits.len(),
                n == bm.bits.len(),
                result.bits@.len() == i,
                result.view().len() == i * 64,
                forall|k: int| 0 <= k && k < i * 64 ==> result.view()[k] == (self.view()[k] || bm.view()[k]),
            decreases n - i
        {
            res_bits = result.bits;
            proof {
                assert(i < self.bits@.len());
                assert(i < bm.bits@.len());
            }
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
                assert_forall_by(|k: int| {
                    requires(0 <= k && k < i * 64);
                    ensures(result.view()[k] == (self.view()[k] || bm.view()[k]));
                    {
                        let chunk = k / 64;
                        if chunk < i - 1 {
                        } else {
                            let bit_index = (k % 64) as u64;
                            assert(get_bit64!(result.bits@[chunk], bit_index)
                                == get_bit64!(self.bits@[chunk], bit_index)
                                || get_bit64!(bm.bits@[chunk], bit_index));
                        }
                    }
                });
            }
        }

        proof {
            assert(result.bits@.len() == n);
            assert(result.view().len() == n * 64);
            assert(self.view().len() == bm.view().len());
            assert(self.view().len() == n * 64);
            assert_forall_by(|idx: int| {
                requires(0 <= idx && idx < result.view().len());
                ensures(result.view()[idx] == (self.view()[idx] || bm.view()[idx]));
                {
                    let chunk = idx / 64;
                    assert(chunk < n);
                }
            });
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
