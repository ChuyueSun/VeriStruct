use vstd::prelude::*;

verus! {

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, usize);

    closed spec fn view(&self) -> (Seq<T>, usize) {
        let contents = if self.head <= self.tail {
            self.ring@.subrange(self.head, self.tail)
        } else {
            self.ring@.subrange(self.head, self.ring.len()) + self.ring@.subrange(0, self.tail)
        };
        (contents, self.ring.len())
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
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring.len() >= 2
        && self.head < self.ring.len()
        && self.tail < self.ring.len()
    }

    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == self.view().0.len(),
    {
        let r = if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        };
        proof {
            if self.head <= self.tail {
                assert(r == self.tail - self.head);
                assert(self.view().0.len() == self.tail - self.head);
            } else if self.tail < self.head {
                assert(r == (self.ring.len() - self.head) + self.tail);
                assert(self.view().0.len() == (self.ring.len() - self.head) + self.tail);
            } else {
                assert(r == 0);
                assert(self.view().0.len() == 0);
            }
        }
        r
    }

    pub fn has_elements(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().0.len() > 0),
    {
        let b = self.head != self.tail;
        proof {
            assert(b == (self.view().0.len() > 0));
        }
        b
    }

    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().0.len() == self.ring.len() - 1),
    {
        let f = self.head == ((self.tail + 1) % self.ring.len());
        proof {
            if f {
                assert(self.view().0.len() == self.ring.len() - 1);
            } else {
                assert(self.view().0.len() <= self.ring.len() - 1);
            }
        }
        f
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() >= 2,
        ensures
            ret.inv(),
            ret@.0 == seq![],
            ret@.1 == ring.len(),
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            succ == !old(self).is_full(),
            if succ {
                self.view().0 == old(self).view().0.add(val)
            } else {
                self.view().0 == old(self).view().0
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(!old(self).is_full());
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            match ret {
                Some(v) => {
                    old(self).has_elements()
                    && v == old(self).view().0.index(0)
                    && self.view().0 == old(self).view().0.drop(1)
                },
                None => {
                    !old(self).has_elements()
                    && self.view().0 == old(self).view().0
                }
            },
    {
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
            self.inv(),
            ret == self.ring.len() - 1 - self.view().0.len(),
    {
        let a = self.ring.len().saturating_sub(1 + self.len());
        proof {
            assert(a == self.ring.len() - 1 - self.view().0.len());
        }
        a
    }
}

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
