use vstd::prelude::*;

verus! {

    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    closed spec fn ring_seq<T: Copy>(rb: &RingBuffer<T>) -> Seq<T> {
        let n = rb.ring@.len();
        let c = if rb.tail >= rb.head {
            rb.tail - rb.head
        } else {
            n - rb.head + rb.tail
        };
        Seq::new(c, |i| rb.ring@[(rb.head + i) % n])
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            (ring_seq(self), self.ring@.len())
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
                self.inv(),
                ret == self.view().0.len(),
        {
            proof {
                assert(self.view().0.len() == if self.tail >= self.head {
                    self.tail - self.head
                } else {
                    self.ring.len() - self.head + self.tail
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
                self.inv(),
                ret == (self.view().0.len() > 0),
        {
            proof {}
            self.head != self.tail
        }

        /// Returns true if the buffer is full, false otherwise.
        pub fn is_full(&self) -> (ret: bool)
            requires
                self.inv(),
            ensures
                self.inv(),
                ret == (self.view().0.len() == self.ring@.len() - 1),
        {
            proof {}
            self.head == ((self.tail + 1) % self.ring.len())
        }

        /// Creates a new RingBuffer with the given backing `ring` storage.
        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
            requires
                ring.len() > 0,
            ensures
                ret.inv(),
                ret.view().0.len() == 0,
                ret.view().1 == ring@.len(),
        {
            RingBuffer {
                head: 0,
                tail: 0,
                ring,
            }
        }

        /// This method attempts to add a new element to the back of the ring buffer.
        ///
        /// The success of this operation is directly determined by the buffer's capacity:
        /// - If the buffer is **not full**, the element is added and the method returns `true`
        /// - If the buffer is **full**, no element is added and the method returns `false`
        ///
        /// # Arguments
        /// * `val` - The value to add to the buffer
        ///
        /// # Returns
        /// * `true` - The element was successfully added (buffer was not full)
        /// * `false` - The element could not be added (buffer was full)
        pub fn enqueue(&mut self, val: T) -> (succ: bool)
            requires
                self.inv(),
            ensures
                self.inv(),
                succ == !old(self).is_full(),
                if succ {
                    self.view().0 == old(self).view().0.add(old(self).view().0.len(), val)
                } else {
                    self.view() == old(self).view()
                },
        {
            if self.is_full() {
                false
            } else {
                proof {}
                my_set(&mut self.ring, self.tail, val);
                self.tail = (self.tail + 1) % self.ring.len();
                true
            }
        }

        /// Removes and returns the front element from the ring buffer.
        ///
        /// The success of this operation is directly determined by the buffer's contents:
        /// - If the buffer is **not empty**, the front element is removed and returned as `Some(T)`
        /// - If the buffer is **empty**, no element is removed and the method returns `None`
        ///
        /// # Returns
        /// * `Some(T)` - The front element if the buffer was not empty
        /// * `None` - If the buffer was empty
        pub fn dequeue(&mut self) -> (ret: Option<T>)
            requires
                self.inv(),
            ensures
                self.inv(),
                ret.is_Some() == old(self).has_elements(),
                if ret.is_Some() {
                    ret.get_Some_0() == old(self).view().0.index(0)
                    && self.view().0 == old(self).view().0.subrange(1, old(self).view().0.len())
                } else {
                    self.view() == old(self).view()
                },
        {
            proof {}
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
                ret == self.ring@.len() - 1 - self.view().0.len(),
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
