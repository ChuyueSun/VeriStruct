/// This file provides a ring buffer implementation along with some utility specs,
/// and it demonstrates how to implement the View trait for the RingBuffer struct.
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

// ========== VIEW IMPLEMENTATION ==========
impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let ring_seq = self.ring@;
        let ring_len_i = ring_seq.len() as int;
        let head_i = self.head as int;
        let tail_i = self.tail as int;
        let length_i = if tail_i >= head_i {
            tail_i - head_i
        } else {
            ring_len_i - (head_i - tail_i)
        };
        let length = (length_i as nat);
        let content = Seq::new(length, |i: int| {
            ring_seq[((head_i + i) % ring_len_i) as int]
        });
        (content, ( ring_seq.len() - 1 ) as nat)
    }
}
// ========================================

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
        &&& self.ring.len() >= 2
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // (no preconditions needed for &self)
        ensures
            ret == self@.0.len()
    {
        let raw = if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        };
        proof {
            // PROOF BLOCK FOR len:
            let ring_seq = self.ring@;
            let ring_len_i = ring_seq.len() as int;
            let head_i = self.head as int;
            let tail_i = self.tail as int;
            let length_i = if tail_i >= head_i {
                tail_i - head_i
            } else {
                ring_len_i - (head_i - tail_i)
            };
            // Show that raw == length_i
            assert((tail_i >= head_i) ==> (raw as int == tail_i - head_i));
            assert((tail_i < head_i)  ==> (raw as int == ring_len_i - (head_i - tail_i)));
            // Now by cases either tail_i >= head_i or tail_i < head_i, we conclude raw == length_i
            assert(raw as int == length_i);
            // By definition of self@, the .0 is the content with length == length_i
            assert(ret == raw);
            assert(raw == self@.0.len());
        }
        raw
    }

    /// Returns true if there are any items in the buffer, false otherwise.
    pub fn has_elements(&self) -> (ret: bool)
        requires
            // (no preconditions needed for &self)
        ensures
            ret <==> (self@.0.len() > 0)
    {
        proof {
            // PROOF BLOCK FOR has_elements:
            // self@.0.len() > 0 <==> self.head != self.tail
            let ring_seq = self.ring@;
            let head_i = self.head as int;
            let tail_i = self.tail as int;
            if self.head != self.tail {
                assert(self@.0.len() > 0);
            } else {
                assert(self@.0.len() == 0);
            }
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // (no preconditions needed for &self)
        ensures
            ret <==> self@.0.len() == self@.1
    {
        proof {
            // PROOF BLOCK FOR is_full:
            // self@.0.len() == ring_seq.len() - 1  <==>  head == (tail+1)%ring.len()
            let ring_seq = self.ring@;
            let ring_len_i = ring_seq.len() as int;
            let head_i = self.head as int;
            let tail_i = self.tail as int;
            let buffer_len = if tail_i >= head_i {
                tail_i - head_i
            } else {
                ring_len_i - (head_i - tail_i)
            };
            // The spec says self@.0.len() == ring_seq.len() - 1  <==> ret
            // ret is (self.head == (self.tail+1) % self.ring.len())
            // We connect these conditions.
            if self.head == ((self.tail + 1) % self.ring.len()) {
                assert(buffer_len as int == ring_len_i - 1);
            } else {
                assert(buffer_len as int != ring_len_i - 1);
            }
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() >= 2
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len() - 1
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to enqueue a new element.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self)@.0.len() <= old(self)@.1
        ensures
            self@.1 == old(self)@.1,
            succ ==> self@.0 == old(self)@.0.push(val),
            !succ ==> self@.0 =~= old(self)@.0,
            succ <==> old(self)@.0.len() < old(self)@.1
    {
        if self.is_full() {
            false
        } else {
            proof {
                // PROOF BLOCK FOR enqueue:
                // We'll show that after my_set, ring is updated at tail index,
                // and that new tail => new self@.0 = old(self)@.0.push(val).
                assert(old(self)@.0.len() < old(self)@.1);
            }
            my_set(&mut self.ring, self.tail, val);
            proof {
                // After the actual modification, use assert_seqs_equal! to confirm ring@ is updated.
                assert_seqs_equal!(
                    self.ring@,
                    old(self).ring@.update(self.tail as int, val)
                );
                // Next, show that self@.0 == old(self)@.0.push(val).
                let old_content_len = old(self)@.0.len();
                // old(self)@.0 is the sequence of length old_content_len,
                // the new element is placed exactly at 'old tail' position,
                // and the new tail is (old tail + 1) mod ring.len().
                // That effectively appends to the content at the "end" in ring-buffer sense.
                assert(self@.0.len() == old_content_len + 1);
            }
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes the front element, if any.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self)@.0.len() <= old(self)@.1
        ensures
            self@.1 == old(self)@.1,
            ret.is_Some() <==> old(self)@.0.len() > 0,
            ret.is_Some() ==> ret.get_Some_0() == old(self)@.0[0],
            ret.is_Some() ==> self@.0 =~= old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int),
            ret.is_None()  ==> self@.0 =~= old(self)@.0
    {
        proof {
            // PROOF BLOCK FOR dequeue:
            // We'll show if old(self)@.0.len() > 0, we remove the first element
            // => ret has that element, and the new content is subrange(1,..).
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            proof {
                // We haven't changed ring except for head movement (the ring Vec is unchanged).
                // So ring@ is unchanged, but the content shifts by 1.
                // ret is Some(val).
                // The new content is old(self)@.0.subrange(1, ...).
            }
            Some(val)
        } else {
            None
        }
    }

    /// Returns how many more elements can be enqueued before full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            // (no preconditions needed for &self)
        ensures
            (ret as int) == (self@.1 - self@.0.len())
    {
        let raw = self.ring.len().saturating_sub(1 + self.len());
        proof {
            // PROOF BLOCK FOR available_len:
            // self@.1 = ring@.len() - 1
            // self@.0.len() = number_of_elements
            // so ret = (ring.len() - (1 + number_of_elements)) -> ring.len() -1 - number_of_elements
            // which matches self@.1 - self@.0.len().
            let ring_seq = self.ring@;
            let capacity_minus_one = (ring_seq.len() - 1) as int;
            let used = self@.0.len();
            assert((raw as int) == capacity_minus_one - used);
        }
        raw
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
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
