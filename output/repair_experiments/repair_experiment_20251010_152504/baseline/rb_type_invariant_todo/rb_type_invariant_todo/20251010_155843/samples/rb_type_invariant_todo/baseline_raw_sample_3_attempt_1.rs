use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        (self.contents(), self.ring.len())
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
    closed spec fn contents(&self) -> Seq<T> {
        let r = self.ring@;
        if self.tail >= self.head {
            r.subrange(self.head, self.tail)
        } else {
            r.subrange(self.head, r.len()) + r.subrange(0, self.tail)
        }
    }

    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.head < self.ring.len()
        && self.tail < self.ring.len()
        && self.ring.len() > 0
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self@.0.len(),
    {
        proof {
            reveal(self.contents());
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
            ret == (self@.0.len() > 0),
    {
        proof {
            reveal(self.contents());
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self@.0.len() == self.ring.len() - 1),
    {
        proof {
            reveal(self.contents());
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0,
        ensures
            ret.inv(),
            ret.head == 0,
            ret.tail == 0,
            ret.ring@ == ring@,
            ret@.0 == Seq::<T>::empty(),
            ret@.1 == ring@.len(),
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Enqueues a new element to the back of the ring buffer.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            if old(self).is_full() {
                !succ
                && self@ == old(self)@
            } else {
                succ
                && self@.0 == old(self)@.0.add(val)
                && self@.1 == old(self)@.1
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                reveal(self.contents());
                reveal_old(self.contents());
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Dequeues the front element from the ring buffer.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            match ret {
                Some(x) => old(self).has_elements()
                           && x == old(self)@.0.index(0)
                           && self@.0 == old(self)@.0.subrange(1, old(self)@.0.len())
                           && self@.1 == old(self)@.1,
                None => !old(self).has_elements() && self@ == old(self)@
            },
    {
        proof {
            reveal(self.contents());
            reveal_old(self.contents());
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
            self.inv(),
            ret == self.ring.len() - 1 - self@.0.len(),
    {
        proof {
            reveal(self.contents());
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
