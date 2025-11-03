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
        let n = self.ring@.len();
        let h = self.head as int;
        let t = self.tail as int;
        let cont = if t >= h {
            self.ring@.subrange(h, t)
        } else {
            self.ring@.subrange(h, n) + self.ring@.subrange(0, t)
        };
        (cont, (n - 1) as nat)
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
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        (self.ring.len() >= 1)
        && (self.head < self.ring.len())
        && (self.tail < self.ring.len())
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv(),
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
            self.inv(),
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
            self.inv(),
        ensures
            ret <==> self.view().0.len() == self.view().1,
    {
        proof {
            // TODO: add proof
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() >= 1,
        ensures
            ret.inv(),
            ret.view().0.len() == 0,
            ret.view().1 == ring.len() - 1,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer.
    /// - If the buffer is not full, returns true and adds the element.
    /// - If the buffer is full, returns false and does not modify contents.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if old(self).view().0.len() == old(self).view().1 {
                succ == false && self@ == old(self)@
            } else {
                succ == true
                && self.view().0 =~= old(self).view().0 + seq![val]
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

    /// Removes and returns the front element from the ring buffer, if it is not empty.
    /// - If the buffer is not empty, returns Some(front element).
    /// - If the buffer is empty, returns None.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if old(self).view().0.len() == 0 {
                ret == None::<T>
                && self@ == old(self)@
            } else {
                ret.is_Some()
                && get_Some_0(ret) == old(self).view().0[0]
                && self.view().0 =~= old(self).view().0.subrange(1, old(self).view().0.len())
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
            self.inv(),
        ensures
            ret == self.view().1 - self.view().0.len(),
    {
        proof {
            // TODO: add proof
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

pub fn main() {
}
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
