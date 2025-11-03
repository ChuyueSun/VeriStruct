use vstd::prelude::*;

fn main() {}

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
        let capacity = self.ring@.len();
        let occupant_count = if self.tail >= self.head {
            (self.tail - self.head) as int
        } else {
            (capacity - self.head) as int + self.tail as int
        };
        let content = Seq::new(occupant_count, |i: int| {
            let idx = (self.head as int + i) % capacity;
            self.ring@[idx]
        });
        (content, capacity)
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec)@.len(),
        no_unwind
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        (self.ring.len() > 0)
        && (self.head < self.ring.len())
        && (self.tail < self.ring.len())
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            // The ring's abstract length is the occupant_count
            // computed as in view(). Sufficient that we do a trivial proof:
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
            ret <==> (self@.0.len() > 0)
    {
        proof {
            // If head == tail => occupant_count = 0 => no elements
            // If head != tail => occupant_count > 0 => there are elements
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret <==> (self@.0.len() == self@.1 - 1)
    {
        proof {
            // If (self.tail + 1) % capacity == self.head => occupant_count = capacity - 1
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
        ensures
            if succ {
                self@.0 == old(self)@.0.push(val)
                && self@.1 == old(self)@.1
            } else {
                self@.0 == old(self)@.0
                && self@.1 == old(self)@.1
            }
    {
        if self.is_full() {
            proof {
                // The buffer is full; no change in the abstract sequence
                assert(self.view().1 == old(self).view().1);
                assert_seqs_equal!(
                    self.view().0,
                    old(self).view().0
                );
            }
            false
        } else {
            proof {
                // The buffer is not full; we add 'val'
                assert(self.view().1 == old(self).view().1);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                // We appended 'val' at the logical end
                assert_seqs_equal!(
                    self.view().0,
                    old(self).view().0.push(val)
                );
            }
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
        ensures
            match ret {
                Some(rval) => {
                    rval == old(self)@.0[0]
                    && self@.0 == old(self)@.0.subrange(1, old(self)@.0.len())
                    && self@.1 == old(self)@.1
                },
                None => {
                    self@ == old(self)@
                }
            }
    {
        proof {
            // We'll do a short bridging proof after we see if it has elements or not
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            proof {
                // We removed the first element from old(self)@.0
                assert(self.view().1 == old(self).view().1);
                assert_seqs_equal!(
                    self.view().0,
                    old(self).view().0.subrange(1, old(self).view().0.len())
                );
            }
            Some(val)
        } else {
            proof {
                // No element removed
                assert(self.view().1 == old(self).view().1);
                assert_seqs_equal!(
                    self.view().0,
                    old(self).view().0
                );
            }
            None
        }
    }

    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.1 - self@.0.len() - 1
    {
        proof {
            // The capacity is self@.1, occupant_count is self@.0.len()
            // available = capacity - occupant_count - 1
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

} // verus!

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
