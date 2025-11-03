use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, int);

    closed spec fn view(&self) -> Self::V {
        (
            if self.tail >= self.head {
                self.ring@.subrange(self.head as int, self.tail as int)
            } else {
                self.ring@.subrange(self.head as int, self.ring@.len())
                    + self.ring@.subrange(0, self.tail as int)
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
    {
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring@.len() > 0
        && self.head < self.ring.len()
        && self.tail < self.ring.len()
    }

    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret == self@.0.len(),
    {
        proof {
            assert(ret == if self.tail >= self.head {
                (self.tail - self.head) as int
            } else {
                (self.ring.len() - self.head + self.tail) as int
            });
            assert(ret == self@.0.len());
        }
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    pub fn has_elements(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            ret == (self@.0.len() > 0),
    {
        proof {
            assert(ret == (self.head != self.tail));
            assert(ret == (self@.0.len() > 0));
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            ret == (self@.0.len() == self.ring@.len() - 1),
    {
        proof {
            assert(ret == (self.head == ((self.tail + 1) % self.ring.len())));
            assert(ret == (self@.0.len() == self.ring@.len() - 1));
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

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

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            self@.1 == old(self)@.1,
            if succ {
                self@.0 == old(self)@.0 + [val]
            } else {
                self@.0 == old(self)@.0
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(old(self)@.0 == self@.0);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            self@.1 == old(self)@.1,
            match ret {
                Some(v) =>
                    v == old(self)@.0.index(0)
                    && self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()),
                None =>
                    self@.0 == old(self)@.0
            },
    {
        proof {
            if self.has_elements() {
                assert(old(self)@.0.len() > 0);
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

    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            ret as int == self.ring@.len() - 1 - self@.0.len(),
    {
        proof {
            assert(ret as int == (self.ring@.len() - 1) - self@.0.len());
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
