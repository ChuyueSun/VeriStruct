use vstd::prelude::*;

verus! {

/// The logical (abstract) view of the ring buffer is a pair:
/// (the sequence of elements currently stored in the buffer, the capacity of the buffer).
impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    /// Returns the sequence of stored elements (front to back) and the capacity as the view.
    closed spec fn view(&self) -> Self::V {
        (self.spec_contents(), self.ring@.len())
    }
}

impl<T: Copy> RingBuffer<T> {
    /// Returns the sequence of stored elements (front to back).
    closed spec fn spec_contents(&self) -> Seq<T> {
        if self.tail >= self.head {
            self.ring@.subrange(self.head, self.tail)
        } else {
            self.ring@.subrange(self.head, self.ring@.len())
                + self.ring@.subrange(0, self.tail)
        }
    }

    /// Invariant for the ring buffer:
    /// 1) The ring storage is non-empty
    /// 2) head and tail pointers are in valid range
    /// 3) Everything is well-formed
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring@.len() > 0
        && self.head < self.ring@.len()
        && self.tail < self.ring@.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self@.0.len(),
    {
        proof { }
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
            ret == (self@.0.len() > 0),
            ret == (self.head != self.tail),
    {
        proof { }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.head == ((self.tail + 1) % self.ring.len())),
            // Equivalently, ret == (self@.0.len() == self.ring@.len() - 1)
    {
        proof { }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() > 0,
        ensures
            ret.inv(),
            ret@.0.len() == 0,
            ret@.1 == ring@.len(),
            ret.head == 0,
            ret.tail == 0,
            ret.ring@ == ring@,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Tries to enqueue (add) the value `val` to the buffer. Returns `true` if successful,
    /// or `false` if the buffer was full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if !old(self).is_full() {
                succ
                && self@.0 == old(self)@.0.add(val)
                && self@.1 == old(self)@.1
            } else {
                !succ
                && self@.0 == old(self)@.0
                && self@.1 == old(self)@.1
            },
    {
        if self.is_full() {
            false
        } else {
            proof { }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Tries to dequeue (remove) the front element of the buffer. Returns `Some(val)` if
    /// successful, or `None` if the buffer was empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            match ret {
                Some(v) => old(self)@.0.len() > 0
                    && v == old(self)@.0.index(0)
                    && self@.0 == old(self)@.0.subrange(1, old(self)@.0.len())
                    && self@.1 == old(self)@.1,
                None => old(self)@.0.len() == 0
                    && self@.0 == old(self)@.0
                    && self@.1 == old(self)@.1,
            },
    {
        proof { }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued before it is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            // The ring can hold at most ring.len() - 1 elements, so available == ring.len() - 1 - current_len
            ret == self.ring.len() - 1 - self@.0.len(),
    {
        proof { }
        self.ring.len().saturating_sub(1 + self.len())
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

pub fn main() {
}
}
