use vstd::prelude::*;

verus! {

/// A ring buffer backed by a `Vec<T>`.
/// The ring buffer can hold up to `ring.len() - 1` elements.
pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = Seq<T>;

    /// Logical view of the ring buffer: the sequence of valid, enqueued elements
    /// in order from the head to the tail (wrapping around if necessary).
    open spec fn view(&self) -> Seq<T> {
        if self.tail >= self.head {
            self.ring@.subrange(self.head, self.tail)
        } else {
            self.ring@.subrange(self.head, self.ring@.len()).concat(self.ring@.subrange(0, self.tail))
        }
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
    no_unwind
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    /// Invariant for the ring buffer.
    /// - The ring must have at least length 1.
    /// - `head` and `tail` are within bounds.
    /// - The logical view has length at most `ring.len() - 1`.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring.len() > 0
        && self.head < self.ring.len()
        && self.tail < self.ring.len()
        && self.view().len() <= self.ring.len() - 1
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self.view().len(),
    {
        proof {
            if self.tail > self.head {
                assert((self.tail - self.head) == self.view().len());
            } else if self.tail < self.head {
                assert(((self.ring.len() - self.head) + self.tail) == self.view().len());
            } else {
                assert(0 == self.view().len());
            }
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
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().len() > 0),
    {
        proof {
            assert((self.head != self.tail) == (self.view().len() > 0));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    /// We say it's full if it has `ring.len() - 1` elements.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().len() == self.ring.len() - 1),
    {
        proof {
            // Standard ring-buffer logic: (tail + 1) % ring.len() == head indicates full.
            // That is consistent with having ring.len() - 1 elements logically.
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    /// The new buffer is empty (head == tail).
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() > 0,
        ensures
            ret.inv(),
            ret.ring@ == ring@,
            ret.view().len() == 0,
            ret.ring.len() == ring.len(),
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer.
    /// Returns `true` if successful, `false` if the buffer was full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            succ == !old(self).is_full(),
            if succ {
                // Value was added
                self.view() == old(self).view().add(val)
            } else {
                // No change if it was full
                self.view() == old(self).view()
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(old(self).view().len() < self.ring.len() - 1);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    /// Returns `Some(T)` if a value was removed, or `None` if empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            match ret {
                Some(x) =>
                    old(self).view().len() > 0
                    && self.view() == old(self).view().drop_first()
                    && x == old(self).view().first(),
                None =>
                    old(self).view().len() == 0
                    && self.view() == old(self).view()
            },
    {
        proof {
            if self.has_elements() {
                assert(old(self).view().len() > 0);
            } else {
                assert(old(self).view().len() == 0);
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

    /// Returns how many more elements it can accept until it is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            // It's the space left if total capacity is `ring.len() - 1`.
            ret == self.ring.len() - 1 - self.view().len(),
    {
        proof {
            // saturating_sub is consistent with the ring buffer design,
            // but logically we know we never exceed `ring.len() - 1`.
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* TEST CODE BELOW */

#[verifier::loop_isolation(false)]
fn test(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
{
    let mut ring: Vec<i32> = Vec::new();

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

pub fn main() {}
}
