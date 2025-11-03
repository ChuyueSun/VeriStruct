use vstd::prelude::*;

verus! {

pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
{
    if (a > b) {
        (a - b) as nat
    } else {
        0
    }
}

#[verifier::external_fn_specification]
pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
ensures
    ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

pub open spec fn mod_auto_plus(n: int) -> bool
    recommends
        n > 0
{
    forall|x: int, y: int|
    {
        let z = (x % n) + (y % n);
        ((0 <= z && z < n && #[trigger] ((x + y) % n) == z)
            || (n <= z && z < 2*n && ((x + y) % n) == z - n))
    }
}

pub open spec fn mod_auto_minus(n: int) -> bool
    recommends
        n > 0
{
    forall|x: int, y: int|
    {
        let z = (x % n) - (y % n);
        ((0 <= z && z < n && #[trigger] ((x - y) % n) == z)
            || (-n <= z && z < 0 && ((x - y) % n) == z + n))
    }
}

pub open spec fn mod_auto(n: int) -> bool
    recommends
        n > 0
{
    &&& (n % n == 0 && (-n) % n == 0)
    &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
    &&& (forall|x: int| 0 <= x && x < n <==> #[trigger] (x % n) == x)
    &&& mod_auto_plus(n)
    &&& mod_auto_minus(n)
}

pub proof fn lemma_mod_auto(n: int)
    requires
        n > 0
    ensures
        mod_auto(n)
{
    admit()
}

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let ring_view = self.ring@;
        let cap = ring_view.len();
        if self.tail >= self.head {
            (ring_view.subrange(self.head as int, self.tail as int), cap)
        } else {
            (ring_view.subrange(self.head as int, cap as int)
               + ring_view.subrange(0, self.tail as int), cap)
        }
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
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
        self.ring.len() > 0
        && self.head < self.ring.len()
        && self.tail < self.ring.len()
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // Skip self.inv() due to #[verifier::type_invariant].
        ensures
            ret == self@.0.len()
    {
        proof {
            assert(0 <= self.ring.len() - self.head);
            assert(self.ring.len() - self.head < self.ring.len());
            assert(self.tail < self.ring.len());
            assert(((self.ring.len() - self.head) + self.tail) < 2 * self.ring.len());

            if self.tail > self.head {
                assert(self.tail - self.head == self@.0.len());
            } else if self.tail < self.head {
                assert((self.ring.len() - self.head) + self.tail == self@.0.len());
            } else {
                assert(self@.0.len() == 0);
            }
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
            // Skip self.inv() because of #[verifier::type_invariant].
        ensures
            ret == (self@.0.len() > 0)
    {
        proof {
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // Skip self.inv() because of #[verifier::type_invariant].
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0,
            ring.len() > 0,
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len()
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Enqueues a value into the ring buffer if space is available.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self)@.0.len() <= old(self)@.1,
        ensures
            succ ==> self@.0 == old(self)@.0.push(val),
            succ ==> self@.1 == old(self)@.1,
            !succ ==> self@ == old(self)@,
            succ == (old(self)@.0.len() < old(self)@.1 - 1)
    {
        if self.is_full() {
            false
        } else {
            proof {
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Dequeues a value from the ring buffer if one is available.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self)@.0.len() <= old(self)@.1
        ensures
            match ret {
                Some(x) =>
                    self@.0 == old(self)@.0.subrange(1, old(self)@.0.len() as int)
                    && x == old(self)@.0[0],
                None =>
                    self@ == old(self)@
            },
            (ret.is_some()) == (old(self)@.0.len() > 0),
            self@.1 == old(self)@.1
    {
        proof {
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns how many elements can still be added before the ring is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            // Skip self.inv() due to #[verifier::type_invariant].
        ensures
            ret == self@.1 - 1 - self@.0.len()
    {
        proof {
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* TEST CODE BELOW */

#[verifier::loop_isolation(false)]
fn test(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX
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

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
