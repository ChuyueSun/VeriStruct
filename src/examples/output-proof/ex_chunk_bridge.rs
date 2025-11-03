use vstd::prelude::*;
fn main() {}

verus! {
    pub struct Chunked {
        chunks: Vec<u64>,
        len_bits: usize,
    }

    impl Chunked {
        pub closed spec fn view(&self) -> Seq<bool> {
            let n_bits = self.len_bits as int;
            Seq::new(n_bits, |k: int| {
                let chunk = (k / 64) as int;
                let off   = (k % 64) as int;
                if 0 <= chunk && chunk < self.chunks@.len() {
                    get_bit64(self.chunks@[chunk], off as u64)
                } else { false }
            })
        }
    }

    spec fn view_from(chunks: Seq<u64>, len_bits: int) -> Seq<bool> {
        Seq::new(len_bits, |k: int| {
            let chunk = (k / 64) as int;
            let off   = (k % 64) as int;
            if 0 <= chunk && chunk < chunks.len() { get_bit64(chunks[chunk], off as u64) } else { false }
        })
    }

    spec fn combine(a: bool, b: bool) -> bool { a || b }

    proof fn chunk_op_lemma(a: u64, b: u64, r: u64, off: int)
        requires 0 <= off < 64
        ensures get_bit64(r, off as u64) == combine(get_bit64(a, off as u64), get_bit64(b, off as u64))
    { }

    pub fn combine_into(a: &Chunked, b: &Chunked) -> (c: Chunked)
        requires a@.len() == b@.len()
        ensures
            c@.len() == a@.len(),
            forall|k: int| #![auto] 0 <= k < c@.len() ==> c@[k] == combine(a@[k], b@[k])
    {
        let n_chunks = a.chunks.len();
        let mut out_chunks: Vec<u64> = Vec::new();
        out_chunks.reserve(n_chunks);
        let mut i: usize = 0;
        while i < n_chunks
            // ========== INFERRED INVARIANTS ==========
            invariant
                0 <= i as int <= n_chunks as int,
                out_chunks@.len() == i as int,
                a@.len() == b@.len() == a.len_bits as int,
                forall|k: int| #![auto]
                    0 <= k < i as int * 64 ==>
                    view_from(out_chunks@, a.len_bits as int)[k] == combine(a@[k], b@[k]),
            decreases n_chunks as int - i as int
            // =========================================
        {
            let a_chunk = a.chunks[i];
            let b_chunk = b.chunks[i];
            let r_chunk = a_chunk | b_chunk;
            out_chunks.push(r_chunk);

            // ========== INFERRED PROOF ==========
            proof {
                assert forall|off: int| 0 <= off < 64 implies
                    view_from(out_chunks@, a.len_bits as int)[i as int * 64 + off]
                        == combine(a@[i as int * 64 + off], b@[i as int * 64 + off])
                by {
                    chunk_op_lemma(a_chunk, b_chunk, r_chunk, off);
                }
            }
            // ====================================

            i += 1;
        }

        Chunked { chunks: out_chunks, len_bits: a.len_bits }
    }
}
