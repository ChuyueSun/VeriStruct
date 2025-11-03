use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<T> {
    #[spec]
    fn ring_view(ring: Seq<T>, head: nat, tail: nat) -> Seq<T> {
        if head <= tail {
            ring.subrange(head, tail)
        } else {
            ring.subrange(head, ring.len()) + ring.subrange(0, tail)
        }
    }
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    spec fn view(&self) -> Self::V {
        (
            Self::ring_view(self.ring@, self.head, self.tail),
            self.ring@.len() - 1
        )
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
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring@.len() > 1
        && self.head < self.ring@.len()
        && self.tail < self.ring@.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self.view().0.len(),
    {
        proof {
            // Ensures correctness of len calculation
            assert(self.view().0.len() == if self.tail >= self.head {
                self.tail - self.head
            } else {
                (self.ring@.len() - self.head) + self.tail
            } as nat);
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
            ret == (self.view().0.len() > 0),
    {
        proof {
            // Ensures correctness of has_elements
            assert(ret == (self.head != self.tail));
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().0.len() == self.view().1),
    {
        proof {
            // Ensures correctness of is_full
            assert(ret == (self.head == ((self.tail + 1) % self.ring.len())));
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 1,
        ensures
            ret.inv(),
            ret.ring@ == ring@,
            ret.head == 0,
            ret.tail == 0,
            ret.view().0.len() == 0,
            ret.view().1 == ring@.len() - 1,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer, returns success.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if succ {
                self.view().0 == old(self).view().0.add(val)
            } else {
                self.view().0 == old(self).view().0
            },
            self.view().1 == old(self).view().1,
    {
        if self.is_full() {
            false
        } else {
            proof {
                // Show that appending val correlates to updated tail position
                assert(self.view().0.add(val) ==
                       RingBuffer::<T>::ring_view(
                           self.ring@.update(self.tail as int, val),
                           self.head,
                           (self.tail + 1) % self.ring@.len()
                       )
                );
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer, if any.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            match ret {
                Some(v) => self.view().0 == old(self).view().0.drop_first()
                           && v == old(self).view().0.index(0),
                None => self.view().0 == old(self).view().0,
            },
            self.view().1 == old(self).view().1,
    {
        proof {
            // Show removal from the front correlates to updated head position
            if self.has_elements() {
                assert(old(self).view().0.len() > 0);
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

    /// Returns the number of elements that can still be enqueued.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self.view().1 - self.view().0.len(),
    {
        proof {
            // Correlates to capacity - current length
            assert(ret == (self.ring@.len() - 1) - self.view().0.len());
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
