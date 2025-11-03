pub fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        // Added preconditions to link bits' lengths with view lengths
        self.view().len() == bm.view().len(),
        self.view().len() == self.bits@.len() * 64,
        bm.view().len() == bm.bits@.len() * 64,
    ensures
        ret.view().len() == self.view().len(),
        forall|i: int|
            0 <= i && i < ret.view().len() ==>
                ret.view()[i] == (self.view()[i] || bm.view()[i]),
{
    let n: usize = self.bits.len();
    let mut i: usize = 0;
    let mut res_bits: Vec<u64> = Vec::new();
    let mut result = BitMap { bits: res_bits };

    while i < n
        invariant
            0 <= i <= n,
            result.bits@.len() == i,
            // Ensure the partial OR up to index i in the word-array
            forall|j: int| 0 <= j < i ==> result.bits@[j] == (self.bits@[j] | bm.bits@[j]),
            self.bits@.len() == n,
            bm.bits@.len() == n,
            // Link the partial result to the bit-level view
            result.view().len() == i * 64,
            forall|k: int|
                0 <= k < i as int * 64 ==>
                    result.view()[k] == (self.view()[k] || bm.view()[k]),
        decreases n - i
    {
        let mut rb = result.bits;
        let u1: u64 = self.bits[i];
        let u2: u64 = bm.bits[i];
        let or_int: u64 = u1 | u2;

        proof {
            bit_or_64_proof(u1, u2, or_int);
        }

        rb.push(or_int);
        result = BitMap { bits: rb };
        i = i + 1;
    }

    result
}
