#![allow(unused_imports)]
use vstd::prelude::*;
fn main() {}

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
    type V = Seq<T>;

    closed spec fn view(&self) -> Self::V {
        if self.tail >= self.head {
            self.ring@.subrange(self.head as int, self.tail as int)
        } else {
            self.ring@.subrange(self.head as int, self.ring@.len() as int)
                + self.ring@.subrange(0, self.tail as int)
        }
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
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
        &&& (self.ring.len() > 0)
        &&& (self.head < self.ring.len())
        &&& (self.tail < self.ring.len())
    }

    pub fn len(&self) -> (ret: usize)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == self@.len()
    {
        proof {
            // We rely on subrange logic for computing self@.len().
            // The ring buffer length is (tail - head) mod ring.len().
            // Summarize or reference any needed arithmetic lemmas:
            lemma_mod_auto(self.ring.len() as int);
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
            self.inv()
        ensures
            self.inv(),
            ret == (self@.len() > 0)
    {
        proof {
            // head == tail  <==>  subrange is empty => length=0
            // head != tail  <==>  subrange non-empty => length>0
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == (self@.len() == self.ring@.len() - 1)
    {
        proof {
            // The buffer is "full" if it has ring@.len() -1 elements.
            // That occurs exactly when tail == (head + 1) mod ring.len().
            lemma_mod_auto(self.ring.len() as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() > 0
        ensures
            ret.inv(),
            ret@.len() == 0
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv()
        ensures
            self.inv(),
            succ ==> self@.len() == old(self)@.len() + 1,
            !succ ==> self@.len() == old(self)@.len()
    {
        if self.is_full() {
            false
        } else {
            proof {
                // If ring isn't full, length increases by 1, thanks to tail moving by 1.
                // We set one position in ring, but that doesn't alter subrange except
                // to place the new element at the tail, then tail is advanced.
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv()
        ensures
            self.inv(),
            ret.is_some() ==> self@.len() == old(self)@.len() - 1,
            ret.is_none() ==> self@.len() == old(self)@.len()
    {
        proof {
            // If the buffer has elements, we remove the item at head, so length drops by 1.
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
            self.inv()
        ensures
            self.inv(),
            ret == (self.ring@.len() - 1 - self@.len())
    {
        proof {
            // ring@.len() -1 is max capacity
            // self@.len() is how many are in the buffer
            // so available is ring@.len() -1 - self@.len().
            lemma_mod_auto(self.ring.len() as int);
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

}

// Checkpoint Best VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
// Compilation Error: True
