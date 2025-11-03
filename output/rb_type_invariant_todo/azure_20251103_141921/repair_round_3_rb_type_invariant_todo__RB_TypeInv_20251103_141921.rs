#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

impl<T: Copy> RingBuffer<T> {
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            ret.is_some() ==> ret.unwrap() == old(self)@.0[0],
            ret.is_some() ==> self@.0 == old(self)@.0.subrange(1, old(self)@.0.len() as int),
            ret.is_none() ==> ret == None::<T>,
            ret.is_none() ==> self@.0 == old(self)@.0,
            // Added by AI
            ret.is_none() ==> self.head == old(self).head,
            ret.is_none() ==> self.tail == old(self).tail,
            ret.is_none() <==> old(self)@.0.len() == 0,
            ret.is_some() <==> old(self)@.0.len() > 0
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }
}

// Repair Round 3 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
