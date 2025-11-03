use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    #[verifier::spec]
    fn view(&self) -> Self::V {
        (self.content(), self.ring.view().len())
    }
}

#[verifier::spec]
fn seq_from_ring<T: Copy>(ring_seq: Seq<T>, head: usize, tail: usize) -> Seq<T> {
    if tail >= head {
        ring_seq.subrange(head, tail)
    } else {
        ring_seq.subrange(head, ring_seq.len()) + ring_seq.subrange(0, tail)
    }
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::spec]
    fn content(&self) -> Seq<T> {
        seq_from_ring(self.ring.view(), self.head, self.tail)
    }

    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.head < self.ring.view().len()
        && self.tail < self.ring.view().len()
        && 0 < self.ring.view().len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret == self@.0.len(),
    {
        proof {
            if self.tail >= self.head {
                assert(self.content() == self.ring@.subrange(self.head, self.tail));
            } else {
                assert(self.content() ==
                    self.ring@.subrange(self.head, self.ring@.len())
                    + self.ring@.subrange(0, self.tail));
            }
            assert(self.content().len() == if self.tail >= self.head {
                self.tail - self.head
            } else {
                self.ring@.len() - self.head + self.tail
            });
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
            ret == (self@.0.len() > 0),
    {
        proof {
            assert((self.content().len() > 0) == (self.head != self.tail));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            ret == (self@.0.len() == self@.1 - 1),
    {
        proof {
            assert((self@.0.len() == self@.1 - 1)
                == (self.head == ((self.tail + 1) % self.ring@.len())));
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0,
        ensures
            ret.inv(),
            ret@.0.len() == 0,
            ret@.1 == ring@.len(),
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Adds a new element if not full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            succ == !old(self).is_full(),
            succ ==> self@.0 == old(self)@.0 + seq![val],
            !succ ==> self@.0 == old(self)@.0,
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(!old(self).is_full());
                let old_content = old(self).content();
                assert(self.tail < self.ring@.len());
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                assert(self.content() == old(self).content() + seq![val]);
            }
            true
        }
    }

    /// Removes and returns the front element if not empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret.is_Some() == old(self).has_elements(),
            ret.is_Some() ==> ret.get_Some_0() == old(self)@.0.index(0),
            ret.is_Some() ==> self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()),
            ret.is_None() ==> self@.0 == old(self)@.0,
    {
        proof {}
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            proof {
                assert(ret == Some(val));
                assert(val == old(self).content().index(0));
                assert(self.content() == old(self).content().subrange(1, old(self).content().len()));
            }
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret == self@.1 - 1 - self@.0.len(),
    {
        proof {}
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
