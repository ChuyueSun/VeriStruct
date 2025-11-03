#[verifier::external_body]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

pub fn has_elements(&self) -> (ret: bool)
    ensures
        ret == (self@.0.len() > 0)
{
    proof {
        use_type_invariant(&self);
        lemma_mod_auto(self.ring.len() as int);
        let ring_view = self.ring@;
        let c = if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.ring.len() - self.head + self.tail
        };
        assert(c == self@.0.len());
        assert((self.head != self.tail) == (c > 0));
    }
    self.head != self.tail
} // Added by AI

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
