pub fn or(&self, bm: &BitMap) -> (ret: BitMap)
    requires
        self.view().len() == bm.view().len(),
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
            forall|j: int| 0 <= j < i ==> result.bits@[j] == (self.bits@[j] | bm.bits@[j]),
            self.bits@.len() == n,
            bm.bits@.len() == n,
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

    // At this point i == n, so the loop invariants imply that for
    // all 0 <= k < n*64, the bits in result.view() match the OR of self.view() and bm.view().
    result
}

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
