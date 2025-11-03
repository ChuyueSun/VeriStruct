        while i < n
            invariant
                0 <= i as int <= n as int,
                result.bits@.len() == i as int,
                self@.len() == bm@.len(),
                // Strengthen the range below from i*64 to result.bits@.len()*64
                // so that at the end of the loop (when i == n), we cover 0..(n*64).
                forall|k: int|
                    0 <= k < (result.bits@.len() * 64)
                    ==> result.view()[k] == (self.view()[k] || bm.view()[k]),
            decreases (n as int - i as int)
        {
            res_bits = result.bits;

            proof {
                assert(self.bits@.len() == self.bits.len());
                assert(bm.bits@.len() == bm.bits.len());
                assert(i < self.bits.len());
                assert(i < bm.bits.len());
            }

            let u1: u64 = self.bits[i];
            let u2: u64 = bm.bits[i];
            let or_int: u64 = u1 | u2;
            res_bits.push(or_int);
            result = BitMap { bits: res_bits };

            proof {
                bit_or_64_proof(u1, u2, or_int);

                // Prove that the newly added block of 64 bits also match bitwise-or
                assert forall|off: int|
                    0 <= off < 64
                    implies
                        result.view()[i as int * 64 + off]
                        == (self.view()[i as int * 64 + off]
                            || bm.view()[i as int * 64 + off]);

                // Combine old range [0 .. i*64) with new range [i*64 .. (i+1)*64)
                // to show it holds for [0 .. (i+1)*64).
                // This assertion re-establishes the invariant for the next iteration.
                assert forall|k: int| 0 <= k < ((i as int + 1) * 64)
                    implies result.view()[k] == (self.view()[k] || bm.view()[k]);
            }

            i = i + 1;
        }

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
