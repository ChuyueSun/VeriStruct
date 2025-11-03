#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

impl<T: Copy> RingBuffer<T> {
    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len(),
            // Added by AI
            ((ret == 0) <==> (self.head == self.tail))
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
            // Added by AI
            assert((self@.0.len() == 0) <==> (self.head == self.tail));
        }
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    pub fn has_elements(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() > 0),
            // Added by AI
            ret <==> (self.head != self.tail)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
            // Added by AI
            assert((self@.0.len() > 0) <==> (self.head != self.tail));
        }
        self.head != self.tail
    }
}

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
