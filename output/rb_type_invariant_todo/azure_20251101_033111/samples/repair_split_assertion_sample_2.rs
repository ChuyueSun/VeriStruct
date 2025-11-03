#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            true,
        ensures
            self@.1 == old(self)@.1,
            ret.is_Some() ==> (
                self@.0 == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                && ret.get_Some_0() == old(self)@.0.index(0)
            ),
            ret.is_None() ==> self@.0 == old(self)@.0,
            old(self)@.0.len() == 0 ==> (ret.is_None() && self@.0.len() == 0), // Added by AI
    {
        proof {
            use_type_invariant(&*self);
            if old(self)@.0.len() == 0 {
                // If the ring was empty, we remain empty and ret must be None
            }
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
