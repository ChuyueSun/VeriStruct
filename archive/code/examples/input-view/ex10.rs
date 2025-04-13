pub struct BitMap {
    bits: Vec<u64>,
}

impl View for BitMap {
    // TODO: implement this.
}

impl BitMap {
    fn from(v: Vec<u64>) -> BitMap {
        BitMap { bits: v }
    }

    fn get_bit(&self, index: u32) -> (bit: bool)
        requires
            index < self@.len(),
        ensures
            bit == self@[index as int],
    {
        // REVIEW: at this moment, usize is assumed to be 32 or 64.
        // Therefore, if `index` is u64, verification fails due to the possibility of truncation
        // when we begin to consider `usize` smaller than 32, this might fail again.
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bucket: u64 = self.bits[seq_index];
        get_bit64_macro!(bucket, bit_index as u64)
    }

    fn set_bit(&mut self, index: u32, bit: bool)
        requires
            index < old(self)@.len(),
        ensures
            self@ == old(self)@.update(index as int, bit),
    {
        // REVEIW: Same problem here with above regarding `usize`.
        let seq_index: usize = (index / 64) as usize;
        let bit_index: u32 = index % 64;
        let bv_old: u64 = self.bits[seq_index];
        let bv_new: u64 = set_bit64_macro!(bv_old, bit_index as u64, bit);
        proof {
            set_bit64_proof(bv_new, bv_old, bit_index as u64, bit);
        }
        ;
        self.bits.set(seq_index, bv_new);
        proof {
            assert_seqs_equal!(
                self.view(),
                old(self).view().update(index as int, bit)
            );
        }
        ;
    }

    // bitwise-OR for bitmap
    fn or(&self, bm: &BitMap) -> (ret: BitMap)
        requires
            self@.len() == bm@.len(),
        ensures
            self@.len() == ret@.len(),
            forall|i: int| 0 <= i < ret@.len() ==> ret@[i] == (self@[i] || bm@[i]),
    {
        let n: usize = self.bits.len();
        let mut i: usize = 0;
        let mut res_bits: Vec<u64> = Vec::new();
        let mut result = BitMap { bits: res_bits };
        while i < n
            invariant
                i <= n,
                n == self.bits@.len(),
                n == bm.bits@.len(),
                i == result.bits.len(),
                forall|k: int|
                    0 <= k < i ==> or_u64_relation(self.bits@[k], bm.bits@[k], result.bits@[k]),
                forall|k: int| 0 <= k < i * 64 ==> result@[k] == (self@[k] || bm@[k]),
        {
            res_bits = result.bits;
            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            proof {
                bit_or_64_view_proof(u1, u2, or_int);
            }
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };
            i = i + 1;
        }
        result
    }
}
