/// Top-level doc comment stays here

use vstd::prelude::*;

verus! {

pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
{
    if a > b {
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
                ||(n <= z && z < n + n&& ((x + y) % n) == z - n))
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
                ||(-n <= z && z < 0&& ((x - y) % n) == z + n))
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
        let ring_seq = self.ring@;
        let length =
            if self.tail >= self.head {
                (self.tail - self.head) as int
            } else {
                (self.tail + self.ring.len() - self.head) as int
            };
        let queue_seq = Seq::new(length as nat, |i: int| {
            ring_seq[((self.head as int + i) % (ring_seq.len()) as int) as int]
        });
        (queue_seq, ring_seq.len() as nat)
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
        let l = self.ring.len();
        &&& l > 0
        &&& (self.head as int) < l
        &&& (self.tail as int) < l
        &&& if self.tail >= self.head {
            ((self.tail as int) - (self.head as int)) < l
        } else {
            ((l - (self.head as int)) + (self.tail as int)) < l
        }
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == self@.0.len()
    {
        proof {
            // Prove that ret matches self@.0.len() based on the head/tail relationship
            assert(self.tail > self.head) ==> self@.0.len() == (self.tail - self.head) as int;
            assert(self.tail < self.head) ==> self@.0.len() == (self.ring.len() - self.head + self.tail) as int;
            assert(self.tail == self.head) ==> self@.0.len() == 0;
            assert(self.inv());
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
            self.inv()
        ensures
            self.inv(),
            ret == (self@.0.len() > 0)
    {
        proof {
            // If head != tail, then self@.0.len() > 0; otherwise 0
            assert(self.head != self.tail) ==> self@.0.len() > 0;
            assert(self.head == self.tail) ==> self@.0.len() == 0;
            assert(self.inv());
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            let capacity = self@.1;
            // If head == (tail + 1) % ring.len(), then # of elements = ring.len() - 1 => capacity - 1
            // Otherwise, it's not that size.
            assert(self.inv());
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() > 0
        ensures
            ret.inv(),
            ret@.0.len() == 0,
            ret@.1 == ring@.len()
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the ring buffer.
    /// Returns true if successful, false if the buffer was full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv()
        ensures
            self.inv(),
            if succ {
                &&& self@.0.len() == old(self)@.0.len() + 1
                &&& self@.0 =~= old(self)@.0.push(val)
                &&& self@.1 == old(self)@.1
            } else {
                &&& self@ == old(self)@
            }
    {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                assert_seqs_equal!(self@.0, old(self)@.0.push(val));
                assert(self@.1 == old(self)@.1);
                assert(self.inv());
            }
            true
        }
    }

    /// Removes and returns the front element if the buffer is not empty, otherwise None.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv()
        ensures
            self.inv(),
            if ret.is_some() {
                &&& self@.0.len() == old(self)@.0.len() - 1
                &&& ret.get_Some_0() == old(self)@.0[0]
                &&& self@.0 =~= old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                &&& self@.1 == old(self)@.1
            } else {
                &&& self@ == old(self)@
            }
    {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            proof {
                assert(self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()));
                assert(self.inv());
            }
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == (self@.1 - 1 - self@.0.len())
    {
        let x = self.ring.len().saturating_sub(1 + self.len());
        proof {
            // The ensures says ret == self@.1 - 1 - self@.0.len().
            // We know self@.1 == self.ring.len() as nat.
            // saturating_sub(1 + self.len()) matches (ring.len() - 1 - self@.0.len()) unless it hits 0 below.
            // But that lines up with the spec given that we never exceed ring.len() in actual usage.
            assert(x as int == self@.1 as int - 1 - self@.0.len());
            assert(self.inv());
        }
        x
    }
}

/* TEST CODE BELOW */

#[verifier::loop_isolation(false)]
fn test(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX
{
    let mut ring: Vec<i32>= Vec::new();

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

} // verus!

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
