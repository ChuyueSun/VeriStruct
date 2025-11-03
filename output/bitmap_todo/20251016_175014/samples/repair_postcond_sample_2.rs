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

pub struct BitMap {
    bits: Vec<u64>,
}

impl BitMap {
    spec fn view(&self) -> (nat, Set<nat>) {
        let bits_seq = self.bits@;
        let len_bits = bits_seq.len() * 64;
        let set_bits = Set::new(|i: nat|
            i < len_bits && ({
                let chunk = bits_seq[( i / 64 ) as int];
                let offset = i % 64;
                ((chunk >> offset) & 0x1) == 1
            })
        );
        (len_bits, set_bits)
    }

    fn from(v: Vec<u64>) -> (ret: BitMap)
        ensures
            ret.view().0 == (v@.len() * 64),
            forall|i: nat|
                i < v@.len() * 64 ==>
                ret.view().1.contains(i)
                == (((v@[(i / 64) as int] >> (i % 64)) & 0x1) == 1),
    {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            (index as nat) < self.view().0,
        ensures
            bit == self.view().1.contains(index as nat),
    {
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        let bit = get_bit64_macro!(bucket, bit_index as u64);
        proof {
            let v = self.view();
            let bits_seq = self.bits@;
            assert(bits_seq[(index / 64) as int] == bucket);
            // Show that 'bit' matches whether the set in view() contains 'index'.
            // By definition, v.1 contains index if ((bucket >> bit_index) & 1) == 1.
            assert(bit == ((bucket >> (bit_index as u64)) & 0x1 == 1));
            assert(bit == v.1.contains(index as nat));
        }
        bit
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            (index as nat) < old(self).view().0,
        ensures
            self.view().0 == old(self).view().0,
            if bit {
                self.view().1 == old(self).view().1.insert(index as nat)
            } else {
                self.view().1 == old(self).view().1.remove(index as nat)
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
            assert forall|k: nat|
                k < old(self).view().0 ==> (
                   (k == index as nat) ==> (
                      (#[trigger] self.view().1.contains(k) == bit)
                      && (old(self).view().1.contains(k) != bit)
                   )
                ) by {
            };
        }
    }

    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self.view().0 == bm.view().0,
        ensures
            ret.view().0 == self.view().0,
            ret.view().1 == self.view().1.union(bm.view().1),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                0 <= i <= n,
                result.bits.len() == i,
                self.bits@.len() == n,
                bm.bits@.len() == n,
                forall|j: int| 0 <= j < i ==> result.bits@.index(j) == (self.bits@.index(j) | bm.bits@.index(j)),
            decreases n - i
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

} // verus!

fn main() {}

// VEval Score: Compilation Error: False, Verified: 5, Errors: 3, Verus Errors: 4
