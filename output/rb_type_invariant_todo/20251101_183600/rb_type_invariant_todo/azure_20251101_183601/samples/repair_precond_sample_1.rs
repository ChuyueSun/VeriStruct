#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

impl<T: Copy> RingBuffer<T> {
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len()
    {
        let ret = RingBuffer {
            head: 0,
            tail: 0,
            ring,
        };
        proof {
            assert(ret@.0.len() == 0);
            assert(ret@.1 == ret.ring@.len());
        }
        ret
    }

    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len()
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
            assert(c as nat == self@.0.len());
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
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
            assert((self.head != self.tail) == (self@.0.len() > 0));
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
            assert((self.head == ((self.tail + 1) % self.ring.len())) == (self@.0.len() == self@.1 - 1));
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn enqueue(&mut self, val: T)
        requires
            self@.0.len() < self@.1 - 1
        ensures
            self@.0.len() == old(self@.0).len() + 1
    {
        proof {
            assert(self.tail < old(self.ring).len());
        } // Added by AI
        my_set(&mut self.ring, self.tail, val);
        self.tail = (self.tail + 1) % self.ring.len();
    }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
