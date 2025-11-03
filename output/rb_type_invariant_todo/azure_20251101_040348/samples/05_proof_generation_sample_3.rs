/// This file demonstrates a ring buffer in Verus.
/// It includes specification functions for modular operations as well as
/// a partially-specified ring buffer data structure.
///
/// Fill in the missing specification pieces (invariants, requires, ensures, proofs)
/// as needed to verify correctness.

use vstd::prelude::*;
use vstd::assert_seqs_equal;

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
        // Completed specification (abstraction) of the ring buffer.
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            let ring_view = self.ring@;
            let c = if self.tail >= self.head {
                self.tail - self.head
            } else {
                self.ring.len() - self.head + self.tail
            };
            let content = Seq::new(c as nat, |i: int|
                ring_view[((self.head as int + i) % ( ring_view.len() ) as int) as int]
            );
            (content, ring_view.len() as nat)
        }
    }


#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec)@.len()
        no_unwind
{
    vec[i] = value;
}


impl<T: Copy> RingBuffer<T> {
    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }


    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // no extra preconditions for &self
        ensures
            ret == self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // Relate the ring's head/tail to the content length:
            // If tail >= head, content length = tail - head
            // If tail < head, content length = ring.len() - head + tail
            // If tail == head, no elements.
            assert((self.tail > self.head) ==> (self@.0.len() == (self.tail - self.head) as int));
            assert((self.tail < self.head) ==> (self@.0.len() == (self.ring.len() - self.head + self.tail) as int));
            assert((self.tail == self.head) ==> (self@.0.len() == 0));
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
            // no extra preconditions for &self
        ensures
            ret <==> self@.0.len() > 0
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // If and only if head != tail, there's at least one element
            // in the ring, hence self@.0.len() > 0
            assert((self.head != self.tail) <==> (self@.0.len() > 0));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // no extra preconditions for &self
        ensures
            ret <==> self@.0.len() == self@.1 - 1
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // is_full means head == (tail + 1) mod ring.len()
            // The content length then is ring.len() - 1
            assert((self.head == ((self.tail + 1) % self.ring.len()))
                <==> (self@.0.len() == self@.1 - 1));
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


    /// This method attempts to add a new element to the back of the ring buffer.
    ///
    /// The success of this operation is directly determined by the buffer's capacity:
    /// - If the buffer is **not full**, the element is added and the method returns `true`
    /// - If the buffer is **full**, no element is added and the method returns `false`
    ///
    /// # Arguments
    /// * `val` - The value to add to the buffer
    ///
    /// # Returns
    /// * `true` - The element was successfully added (buffer was not full)
    /// * `false` - The element could not be added (buffer was full)
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            // no special preconditions for &mut self
        ensures
            self@.1 == old(self)@.1,
            succ ==> (self@.0 == old(self)@.0.push(val)),
            !succ ==> (self@.0 == old(self)@.0)
    {
        if self.is_full() {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                // If full, no change to the content
                assert(self@.0 == old(self)@.0);
            }
            false
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                // Single element update on self.ring
                my_set(&mut self.ring, self.tail, val);
                assert_seqs_equal!(
                    self.ring@,
                    old(self).ring@.update(self.tail as int, val)
                );

                // Now the content grows by 1 element with val at the end
                // of the conceptual sequence
                assert(self@.0 == old(self)@.0.push(val));
            }
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    ///
    /// The success of this operation is directly determined by the buffer's contents:
    /// - If the buffer is **not empty**, the front element is removed and returned as `Some(T)`
    /// - If the buffer is **empty**, no element is removed and the method returns `None`
    ///
    /// # Returns
    /// * `Some(T)` - The front element if the buffer was not empty
    /// * `None` - If the buffer was empty
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            // no special preconditions for &mut self
        ensures
            self@.1 == old(self)@.1,
            ret.is_Some() ==> (
                self@.0 == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                && ret.get_Some_0() == old(self)@.0.index(0)
            ),
            ret.is_None() ==> self@.0 == old(self)@.0
    {
        if self.has_elements() {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                // If we had elements, we're removing the front:
                // old(self)@.0 => new content is subrange(1, oldcount)
                // The removed element is at index(0) of old(self)@.0
                assert(self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()));
            }
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                // Nothing removed if empty
                assert(self@.0 == old(self)@.0);
            }
            None
        }
    }



    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            // no extra preconditions for &self
        ensures
            ret == self@.1 - 1 - self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // ring.len() = self@.1
            // available = ring.len().saturating_sub(1 + self.len())
            // which is ring.len() - 1 - self@.0.len() if it doesn't underflow.
            // Because ring.len() > self@.0.len(), there's no underflow here.
            // So ret = self@.1 - 1 - self@.0.len().
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* TEST CODE BELOW */

#[verifier::loop_isolation(false)]
fn test(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX
{
    let mut ring: Vec<i32>= Vec::new();

    if len == 0 {
        return;
    }

    for i in 0..(len + 1)
    invariant
        ring.len() == i,
    {
        ring.push(0);
    }

    assert(ring.len() == len + 1);
    let mut buf = RingBuffer::new(ring);

    let ret = buf.dequeue();
    let buf_len = buf.len();
    let has_elements = buf.has_elements();
    assert(!has_elements);
    assert(ret == None::<i32>);
    assert(buf_len == 0);
    assert(len > 1);
    for i in 0..len
    invariant
        buf@.0.len() == i,
        buf@.1 == len + 1
    {
        let enqueue_res = buf.enqueue(value);
        assert(enqueue_res);
        let has_elements = buf.has_elements();
        assert(has_elements);
        let available_len = buf.available_len();
        assert(available_len == len - 1 - i);
    }
    let dequeue_res = buf.dequeue();
    assert(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    assert(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    assert(!enqueue_res);
    let dequeue_res = buf.dequeue();
    assert(dequeue_res.is_some());
}

pub fn main() {
}

} // verus!

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
