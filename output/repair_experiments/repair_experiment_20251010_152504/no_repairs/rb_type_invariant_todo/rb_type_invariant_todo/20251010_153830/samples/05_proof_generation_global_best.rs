use vstd::prelude::*;

verus! {

/* -------------------------------------
 * Existing specification and definitions
 * ------------------------------------- */

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

/* -------------------------------------
 * RingBuffer definition
 * ------------------------------------- */

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let ring_len = self.ring@.len();
        let used_len = if self.tail >= self.head {
            (self.tail - self.head) as nat
        } else {
            (ring_len - self.head + self.tail) as nat
        };
        let content = Seq::new(used_len, |i: int| self.ring@[(((self.head as int) + i) % (ring_len as int)) as int]);
        (content, ring_len)
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

/* -------------------------------------
 * RingBuffer implementation
 * ------------------------------------- */

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
            self.inv(),
        ensures
            self.inv(),
            ret == self.view().0.len(),
    {
        proof {
            use_type_invariant(&*self);
            // We often need modular arithmetic facts because view() uses % self.ring.len()
            // to wrap around. Call the lemma:
            lemma_mod_auto(self.ring.len() as int);
            // Here, each case in the code matches exactly the definition of self.view()'s length.
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
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // self.view().0.len() > 0  <==>  self.head != self.tail
            // by the ring invariants and how the head/tail map to length
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv(),
        ensures
            self.inv(),
            ret == (self.view().0.len() == self.view().1 - 1),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // self.view().0.len() == ring.len() - 1 <==> head == (tail + 1) % ring.len()
        }
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

    /// Attempts to add a new element to the back of the ring buffer.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if old(self).view().0.len() == old(self).view().1 - 1 {
                !succ
                && self@ == old(self)@
            } else {
                succ
                && self@.0 =~= old(self)@.0.push(val)
                && self@.1 == old(self)@.1
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                // Writing val to position tail is consistent with appending to self.view(),
                // then incrementing tail mod ring.len() matches the wrap-around semantics.
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            if old(self).view().0.len() > 0 {
                ret.is_Some()
                && ret.get_Some_0() == old(self).view().0[0]
                && self@.0 =~= old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                && self@.1 == old(self)@.1
            } else {
                ret == None::<T>
                && self@ == old(self)@
            },
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Removing the front element at self.head corresponds to
            // dropping the first element of self.view().0, then self.head
            // increments mod ring.len().
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
            ret == self.view().1 - 1 - self.view().0.len(),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Sums up to ring.len() - 1 minus the current usage.
            // self.len() is self.view().0.len(), so ring.len() - 1 - self.len()
            // matches self.view().1 - 1 - self.view().0.len().
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* -------------------------------------
 * Test Code
 * ------------------------------------- */

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
