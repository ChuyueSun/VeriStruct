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
        let ring_len = self.ring@.len();
        let tail_int = self.tail as int;
        let head_int = self.head as int;
        let l = if tail_int >= head_int {
            tail_int - head_int
        } else {
            ring_len - head_int + tail_int
        };
        (
            Seq::new(l as nat, |i: int| self.ring@[((head_int + i) % (ring_len) as int) as int]),
            ring_len
        )
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len()
        no_unwind
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        (self.ring.len() > 0)
        && (self.head < self.ring.len())
        && (self.tail < self.ring.len())
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // type invariant is enforced automatically, so nothing here
        ensures
            ret == self.view().0.len(),
    {
        proof {
            // TODO: add proof
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
            // type invariant is enforced automatically, so nothing here
        ensures
            ret <==> self.view().0.len() > 0,
    {
        proof {
            // TODO: add proof
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // type invariant is enforced automatically, so nothing here
        ensures
            ret <==> self.view().0.len() == self.view().1 - 1,
    {
        proof {
            // TODO: add proof
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0,
        ensures
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
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            // type invariant is enforced automatically
        ensures
            if succ {
                self.view().0 =~= old(self).view().0.push(val)
            } else {
                self.view().0 =~= old(self).view().0
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                // TODO: add proof
            }
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
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            // type invariant is enforced automatically
        ensures
            match ret {
                Some(x) => {
                    self.view().0 =~= old(self).view().0.subrange(1, ( old(self).view().0.len() ) as int)
                    && x == old(self).view().0[0]
                },
                None => self.view().0 =~= old(self).view().0
            },
    {
        proof {
            // TODO: add proof
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
            // type invariant is enforced automatically
        ensures
            ret == self.view().1 - 1 - self.view().0.len(),
    {
        proof {
            // TODO: add proof
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


// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
