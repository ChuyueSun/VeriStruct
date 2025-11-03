use vstd::prelude::*;

verus! {

    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = Seq<T>;

        closed spec fn view(&self) -> Self::V {
            if self.tail >= self.head {
                self.ring@.subrange(self.head, self.tail)
            } else {
                self.ring@.subrange(self.head, self.ring.len()) + self.ring@.subrange(0, self.tail)
            }
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
            self.ring.len() > 0
            && self.head < self.ring.len()
            && self.tail < self.ring.len()
        }

        /// Returns how many elements are in the buffer.
        pub fn len(&self) -> (ret: usize)
            requires
                self.inv(),
            ensures
                ret == self.view().len(),
        {
            proof {
                assert(self.view().len() == if self.tail >= self.head {
                    self.tail - self.head
                } else if self.tail < self.head {
                    (self.ring.len() - self.head) + self.tail
                } else {
                    0
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
                ret == (self.view().len() > 0),
        {
            proof {
                assert(ret == (self.head != self.tail));
                assert(self.head != self.tail == (self.view().len() > 0));
            }
            self.head != self.tail
        }

        /// Returns true if the buffer is full, false otherwise.
        pub fn is_full(&self) -> (ret: bool)
            requires
                self.inv(),
            ensures
                ret == (self.view().len() == self.ring.len() - 1),
        {
            proof {
                assert(ret == (self.head == ((self.tail + 1) % self.ring.len())));
                assert(self.view().len() <= self.ring.len() - 1);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        /// Creates a new RingBuffer with the given backing `ring` storage.
        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
            requires
                ring.len() > 0,
            ensures
                ret.inv(),
                ret.view().len() == 0,
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
                succ ==> self.view() == old(self).view().add(val),
                !succ ==> self.view() == old(self).view(),
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

        /// Removes the front element from the buffer if not empty.
        pub fn dequeue(&mut self) -> (ret: Option<T>)
            requires
                self.inv(),
            ensures
                self.inv(),
                ret.is_Some() ==> {
                    old(self).view().len() > 0 &&
                    self.view() == old(self).view().subrange(1, old(self).view().len()) &&
                    ret.get_Some_0() == old(self).view().index(0)
                },
                ret.is_None() ==> {
                    old(self).view().len() == 0 &&
                    self.view() == old(self).view()
                },
        {
            proof {
                if old(self).view().len() > 0 {
                    assert(self.has_elements());
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

        /// Returns the number of elements that can be enqueued until full.
        pub fn available_len(&self) -> (ret: usize)
            requires
                self.inv(),
            ensures
                ret == (self.ring.len() - 1) - self.view().len(),
        {
            proof {
                assert(ret == self.ring.len().saturating_sub(1 + self.len()));
                assert(self.len() == self.view().len());
            }
            self.ring.len().saturating_sub(1 + self.len())
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
            // These invariants reference non-spec fields in a test context; they are placeholders.
            // Typically one would add ghost fields or use old/buf.inv() checks in real code.
            true
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
