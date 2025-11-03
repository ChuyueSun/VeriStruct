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
        ex_saturating_sub_spec(a as int, b as int) == ret as int
{
    a.saturating_sub(b)
}

pub open spec fn mod_auto_plus(n: int) -> bool
    recommends
        n > 0,
{
    forall|x: int, y: int|
        {
            let z = (x % n) + (y % n);
            ((0 <= z < n && #[trigger] ((x + y) % n) == z)
                || (n <= z < n + n && ((x + y) % n) == z - n))
        }
}

pub open spec fn mod_auto_minus(n: int) -> bool
    recommends
        n > 0,
{
    forall|x: int, y: int|
        {
            let z = (x % n) - (y % n);
            ((0 <= z < n && #[trigger] ((x - y) % n) == z)
                || (-n <= z < 0 && ((x - y) % n) == z + n))
        }
}

pub open spec fn mod_auto(n: int) -> bool
    recommends
        n > 0,
{
    &&& (n % n == 0 && (-n) % n == 0)
    &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
    &&& (forall|x: int| 0 <= x < n <==> #[trigger] (x % n) == x)
    &&& mod_auto_plus(n)
    &&& mod_auto_minus(n)
}

pub proof fn lemma_mod_auto(n: int)
    requires
        n > 0,
    ensures
        mod_auto(n),
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
        let full = self.ring@;
        let capacity = full.len();
        if (self.head <= self.tail) {
            (full.subrange((self.head as int), (self.tail as int)), capacity)
        } else {
            (
                full.subrange((self.head as int), (capacity as int))
                + full.subrange(0, (self.tail as int)),
                capacity
            )
        }
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
        &&& (self.ring.len() > 0)
        &&& (self.head < self.ring.len())
        &&& (self.tail < self.ring.len())
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        ensures
            ret == self.view().0.len(),
    {
        proof {
            // We do a simple case split based on head and tail.
            if self.head == self.tail {
                // Then the view is empty => length 0
                // ret is 0 => matches the view's length 0
            } else if self.head < self.tail {
                // Then the view is subrange [head, tail) => length tail - head
            } else {
                // Then the view is subrange [head, capacity) concat [0, tail)
                // => length (capacity - head) + tail
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
        ensures
            ret == (self.view().0.len() > 0),
    {
        proof {
            // If head != tail => the view's length > 0
            // If head == tail => the view's length = 0
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self.view().0.len() == self.view().1 - 1),
    {
        proof {
            // If head == ((tail + 1) mod capacity), then the buffer is full:
            // that means the contents are of length capacity - 1
            lemma_mod_auto(self.ring.len() as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        ensures
            ret.view().0.len() == 0,
            ret.view().1 == ring.len(),
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Enqueues a value, returning true if it succeeded, false if full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            if succ {
                &&& self.view().0 =~= old(self).view().0.push(val)
            } else {
                &&& self.view().0 =~= old(self).view().0
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                // By setting ring[tail] = val and tail = (tail + 1) % capacity,
                // we extend old(self).view().0 by one element "val".
                lemma_mod_auto(self.ring.len() as int);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Dequeues a value, returning Some(value) if not empty, None if empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            if ret.is_Some() {
                &&& ret == Some(old(self).view().0[0])
                &&& self.view().0 =~= old(self).view().0.subrange(1, (old(self).view().0.len()) as int)
            } else {
                &&& self.view().0 =~= old(self).view().0
            },
    {
        proof {
            // If buffer not empty => old(self).view().0 has at least 1 element.
            // We remove the front => subrange(1, len).
            lemma_mod_auto(self.ring.len() as int);
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns how many more elements can be enqueued until full.
    pub fn available_len(&self) -> (ret: usize)
        ensures
            ret == self.view().1 - 1 - self.view().0.len(),
    {
        proof {
            // The capacity is ring.len(), the used portion is self.len().
            // So available = capacity - 1 - used.
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

// VEval Score: Compilation Error: False, Verified: 2, Errors: 8, Verus Errors: 23
