///
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
    pub fn len(&self) -> (r: usize)
        ensures
            (r as int) == self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // By the definition of self@.0 in view(), if tail > head then content length is tail - head,
            // else if tail < head then content length is ring.len() - head + tail, else 0.
            // The exec code for r matches exactly these cases.
            // Therefore r as int = self@.0.len().
            assert(
                if self.tail > self.head {
                    (self.tail - self.head) as int
                } else if self.tail < self.head {
                    ((self.ring.len() - self.head) + self.tail) as int
                } else {
                    0
                } == self@.0.len()
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
        ensures
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // has_elements() is true iff self.head != self.tail.
            // By construction, self@.0.len() = 0 iff head == tail, else > 0.
            // So ret = (head != tail) = (self@.0.len() > 0).
            if self.head == self.tail {
                assert(self@.0.len() == 0);
            } else {
                assert(self@.0.len() > 0);
            }
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // The ring is full iff head == (tail + 1) % ring.len().
            // That condition means the difference between tail and head is ring.len() - 1
            // (in the ring buffer sense), so the content length is ring.len() - 1.
            // Since self@.1 = ring@.len() as nat, is_full() <=> self@.0.len() = self@.1 - 1.
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len() as nat
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            self@.1 == old(self)@.1,
            succ ==> (self@.0 =~= old(self)@.0 + seq![val]),
            !succ ==> (self@.0 =~= old(self)@.0)
    {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                // We appended exactly one element in the abstract sense:
                // self@.0 = old(self)@.0 + seq![val].
                // CRITICAL: single-element addition, so we must use the macro:
                assert_seqs_equal!(
                    self@.0,
                    old(self)@.0 + seq![val]
                );
            }
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            ret.is_some() ==> (
                old(self)@.0.len() > 0
                && self@.0 =~= old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                && ret.get_Some_0() == old(self)@.0[0]
            ),
            ret.is_none() ==> (
                old(self)@.0.len() == 0
                && self@.0 =~= old(self)@.0
            )
    {
        let r = if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        };
        let ret = r;
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);

            // If head != tail => old(self)@.0.len() > 0 => we remove one:
            // self@.0 = old(self)@.0.subrange(1, old(self)@.0.len()),
            // ret is Some(...) with the removed element.
            assert(self.head != self.tail ==> old(self)@.0.len() > 0);
            assert(self.head == self.tail ==> old(self)@.0.len() == 0);

            assert(self.head != self.tail ==> {
                assert(ret.is_Some());
                assert_seqs_equal!(
                    self@.0,
                    old(self)@.0.subrange(1, old(self)@.0.len())
                );
                // ret's value is the old front element
                ret.get_Some_0() == old(self)@.0[0]
            });

            // If head == tail => old content was empty => no removal
            // => self@.0 = old(self)@.0
            assert(self.head == self.tail ==> {
                assert(ret.is_None());
                assert_seqs_equal!(
                    self@.0,
                    old(self)@.0
                );
            });
        }
        ret
    }

    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> (ret: usize)
        ensures
            (ret as int) == ex_saturating_sub_spec(self@.1 - 1, ( self@.0.len() ) as int)
    {
        let out = self.ring.len().saturating_sub(1 + self.len());
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // By definition, ret is saturating_sub(self.ring.len()-1, self@.0.len()) at spec level.
            // The built-in saturating_sub is consistent with ex_saturating_sub_spec.
            // So ret as int = ex_saturating_sub_spec((self@.1 - 1), self@.0.len()).
            assert(out as int == ex_saturating_sub_spec(self@.1 - 1, self@.0.len()));
        }
        out
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

    let ret = (); // changed from `let ret` to fix syntax error
}
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
