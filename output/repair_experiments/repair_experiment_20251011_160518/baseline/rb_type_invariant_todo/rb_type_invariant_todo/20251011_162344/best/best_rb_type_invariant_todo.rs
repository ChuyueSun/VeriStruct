use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    spec fn view(&self) -> (Seq<T>, nat) {
        let ring_seq = self.ring@;
        let data_seq = if self.tail >= self.head {
            ring_seq.subrange(self.head, self.tail)
        } else {
            ring_seq.subrange(self.head, ring_seq.len()).add(ring_seq.subrange(0, self.tail))
        };
        (data_seq, ring_seq.len())
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
        self.head < self.ring@.len()
        && self.tail < self.ring@.len()
        && self.ring@.len() >= 2
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret == self@.0.len(),
    {
        proof {
            assert(self@.0.len() == if self.tail >= self.head {
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
            assert(self@.0.len() > 0 == (self.head != self.tail));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            ret == (self@.0.len() == self.ring@.len() - 1),
    {
        proof {
            // A ring buffer is full exactly when it holds `ring@.len() - 1` items
            assert(self@.0.len() == self.ring@.len() - 1 == (self.head == ((self.tail + 1) % self.ring.len())));
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() >= 2,
        ensures
            ret.inv(),
            ret@.0 == Seq::empty(),
            ret@.1 == ring@.len(),
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
            self.inv(),
        ensures
            self.inv(),
            succ == !old(self).is_full(),
            succ ==> (self@.0 == old(self)@.0.add(val)),
            !succ ==> (self@.0 == old(self)@.0),
            self@.1 == old(self)@.1,
    {
        if self.is_full() {
            false
        } else {
            proof {
                // If it's not full, we add one element
                assert(self@.0.len() == old(self)@.0.len() + 1);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret.is_some() == old(self).has_elements(),
            ret.is_some() ==> {
                let x = ret.get_Some_0();
                x == old(self)@.0.index(0)
                && self@.0 == old(self)@.0.subrange(1, old(self)@.0.len())
            },
            ret.is_none() ==> (self@.0 == old(self)@.0),
            self@.1 == old(self)@.1,
    {
        proof {
            // If the buffer is not empty, we remove the first item
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret == self.ring@.len() - 1 - self@.0.len(),
    {
        proof {
            assert(ret == self.ring@.len() - 1 - self@.0.len());
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

// Checkpoint Best VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
// Compilation Error: True
