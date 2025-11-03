use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    // We define the logical contents of the buffer as the sequence of
    // elements in FIFO order, plus the total length of the ring storage.
    closed spec fn view(&self) -> Self::V {
        (
            if self.tail >= self.head {
                self.ring@.subrange(self.head, self.tail)
            } else {
                self.ring@.subrange(self.head, self.ring@.len())
                    .add(self.ring@.subrange(0, self.tail))
            },
            self.ring@.len()
        )
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
        self.ring@.len() >= 1
        && self.head < self.ring@.len()
        && self.tail < self.ring@.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            // The result is the number of stored elements, matching the length of the first component of the view
            ret == self@.0.len()
    {
        proof {
            assert(self.inv());
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
            ret == (self@.0.len() > 0)
    {
        proof {
            assert(self.inv());
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.head == (self.tail + 1) % self.ring.len())
    {
        proof {
            assert(self.inv());
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() >= 1,
        ensures
            ret.inv(),
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
    /// Returns `true` if successful, `false` if full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            // If not full originally, succ is true and one new element is added
            (!old(self).is_full()) ==> succ,
            // If full originally, succ is false and no change
            old(self).is_full() ==> !succ,
            // If we did enqueue...
            succ ==> (self@.0.len() == old(self)@.0.len() + 1),
            succ ==> self@.0.ext_equals(old(self)@.0.add(val)),
            // If we did not enqueue...
            !succ ==> (self@.0 == old(self)@.0)
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(self.inv());
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element, or `None` if empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            // If originally had elements, ret is Some(front)
            old(self).has_elements() ==> ret.is_Some(),
            old(self).has_elements() ==> (ret.get_Some_0() == old(self)@.0.index(0)),
            // If originally empty, ret is None
            !old(self).has_elements() ==> ret == None,
            // The new logical content is old minus the front element, if any
            old(self).has_elements() ==> (self@.0 == old(self)@.0.subrange(1, old(self)@.0.len())),
            !old(self).has_elements() ==> (self@.0 == old(self)@.0)
    {
        proof {
            assert(self.inv());
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of remaining spaces for new elements.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self.ring@.len() - 1 - self.len()
    {
        proof {
            assert(self.inv());
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

pub fn main() {
}

}
