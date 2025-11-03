/// Top-level doc comment stays here

use vstd::prelude::*;

verus! {

pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
{
    if (a > b) {
        (a - b) as nat
    } else {
        0
    }
}

#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the sum `x % n + y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x + y) % n`.
    /// (2) It's in the range `[n, 2n)` and equals `(x + y) % n + n`.
    pub open spec fn mod_auto_plus(n: int) -> bool
        recommends
            n > 0
    {
        forall|x: int, y: int|
            {
                let z = (x % n) + (y % n);
                ((0 <= z && z < n && #[trigger] ((x + y) % n) == z)
                    ||(n <= z && z < n + n&& ((x + y) % n) == z - n))
            }
    }

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the difference `x % n - y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x - y) % n`.
    /// (2) It's in the range `[-n, 0)` and equals `(x - y) % n - n`.
    pub open spec fn mod_auto_minus(n: int) -> bool
        recommends
            n > 0
    {
        forall|x: int, y: int|
            {
                let z = (x % n) - (y % n);
                ((0 <= z && z < n && #[trigger] ((x - y) % n) == z)
                    ||(-n <= z && z < 0&& ((x - y) % n) == z + n))
            }
    }

    /// This function states various useful properties about the modulo
    /// operator when the divisor is `n`.
    pub open spec fn mod_auto(n: int) -> bool
        recommends
            n > 0
    {
        &&& (n % n == 0 && (-n) % n == 0)
        &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
        &&& (forall|x: int| 0 <= x && x < n <==> #[trigger] (x % n) == x)
        &&& mod_auto_plus(n)
        &&& mod_auto_minus(n)
    }

    /// Proof of `mod_auto(n)`, which states various useful properties
    /// about the modulo operator when the divisor is the positive
    /// number `n`
    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0
        ensures
            mod_auto(n)
    {
        admit()
    }


    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            let ring_seq = self.ring@;
            let length =
                if self.tail >= self.head {
                    (self.tail - self.head) as int
                } else {
                    (self.tail + self.ring.len() - self.head) as int
                };
            let queue_seq = Seq::new(( length ) as nat, |i: int| {
                ring_seq[((self.head as int + i) % ( ring_seq.len() ) as int) as int]
            });
            (queue_seq, ring_seq.len() as nat)
        }
    }

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len()
        no_unwind
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // (type_invariant implies ring.len() > 0, so no additional pre needed)
        ensures
            ret == self@.0.len()
    {
        proof {
            use_type_invariant(&*self);
            // Bridge the ring's head/tail logic with the queue length in the view
            assert(
                self@.0.len()
                ==
                if self.tail > self.head {
                    (self.tail - self.head) as int
                } else if self.tail < self.head {
                    (self.ring.len() - self.head + self.tail) as int
                } else {
                    0
                }
            );
        }
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    /// Returns true if there are any items in the buffer, false otherwise.
    pub fn has_elements(&self) -> (ret: bool)
        requires
            // (type_invariant implies ring.len() > 0, so no additional pre needed)
        ensures
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&*self);
            // If head != tail, then the queue is non-empty
            assert((self.head != self.tail) == (self@.0.len() > 0));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // (type_invariant implies ring.len() > 0, so no additional pre needed)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&*self);
            // The ring is full if head == (tail+1) mod ring.len()
            // That corresponds to queue length == ring capacity - 1
            assert(
                (self.head == ((self.tail + 1) % ( self.ring.len() ) as int))
                == (self@.0.len() == self@.1 - 1)
            );
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len()
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            // (type_invariant implies old(self)@.1 > 0, so no explicit pre needed)
        ensures
            succ <==> old(self)@.0.len() < old(self)@.1 - 1,
            succ ==> self@.0 == old(self)@.0.push(val),
            !succ ==> self@.0 == old(self)@.0
    {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);

            proof {
                use_type_invariant(&*self);
                assert(old(self)@.0.len() < old(self)@.1 - 1);
                assert(self@.0 == old(self)@.0.push(val));
            }

            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer, if any.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            // (type_invariant implies old(self)@.1 > 0, so no explicit pre needed)
        ensures
            ret.is_some() <==> old(self)@.0.len() > 0,
            ret.is_some() ==> (
                self@.0 =~= old

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
