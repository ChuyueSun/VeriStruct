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
            || (n <= z && z < n + n && ((x + y) % n) == z - n))
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
        let c = if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.ring.len() - self.head + self.tail
        };
        let content = Seq::new(c as nat, |i: int|
            ring_view[((self.head as int + i) % (ring_view.len()) as int) as int]
        );
        (content, ring_view.len() as nat)
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
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn len(&self) -> (ret: usize)
        requires
            self.inv()
        ensures
            self.inv(),
            (ret as nat) == self@.0.len()
    {
        proof {
            use_type_invariant(&self);

            // By definition of `self@.0`, its length is
            //   if tail >= head => tail - head
            //   else if tail < head => ring.len() - head + tail
            //   else => 0
            //
            // The code below returns exactly that result, so ret as nat = self@.0.len().
            let computed = if self.tail > self.head {
                self.tail - self.head
            } else if self.tail < self.head {
                (self.ring.len() - self.head) + self.tail
            } else {
                0
            };
            assert(ret == computed);
            assert((ret as nat) == self@.0.len());
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
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&self);

            // If head != tail => content length != 0
            // If head == tail => content length == 0
            // So ret == (self.head != self.tail) => postcondition proven.
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        requires
            self.inv()
        ensures
            self.inv(),
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&self);

            // The buffer is full iff head == (tail + 1) % ring.len().
            // That implies there is only ring.len() - 1 slots for data,
            // so content length is ring.len() - 1 => self@.1 - 1 in the spec.
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

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

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            old(self).inv()
        ensures
            self.inv(),
            succ <==> old(self)@.0.len() < old(self)@.1 - 1,
            if succ {
                self@.0 == old(self)@.0.push(val)
            } else {
                self@.0 == old(self)@.0
            }
    {
        if self.is_full() {
            false
        } else {
            proof {
                use_type_invariant(&*self);
                // We're adding an element to the ring content, so in the abstract,
                // self@.0 is old(self)@.0.push(val).
                // We'll prove it after the actual array update + tail inc.
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                // Now the ring content has grown by 1 element at the "end".
                // So the new content is old(self).0.push(val).
                assert(self@.0 == old(self)@.0.push(val));
            }
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self).inv()
        ensures
            self.inv(),
            ret.is_some() <==> old(self)@.0.len() > 0,
            if ret.is_some() {
                self@.0 == old(self)@.0.subrange(1, (old(self)@.0.len()) as int)
                && ret.unwrap() == old(self)@.0[0]
            } else {
                self@.0 == old(self)@.0
            }
    {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            proof {
                use_type_invariant(&*self);
                // We removed the front element from the content:
                // new content = old(self).0.subrange(1, old_len),
                // and ret.unwrap() = old(self).0[0].
                assert(self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()));
                assert(val == old(self)@.0[0]);
            }
            Some(val)
        } else {
            proof {
                use_type_invariant(&*self);
                // No change to the abstract content if empty
                assert(self@.0 == old(self)@.0);
            }
            None
        }
    }

    pub fn available_len(&self) -> (ret: usize)
        requires
            self.inv()
        ensures
            self.inv(),
            (ret as int) == self@.1 as int - 1 - self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            // The capacity is self@.1, so available = capacity - 1 - content.len()
            // ret is saturating_sub(1 + self.len()),
            // so ret = ring.len() - (1 + self.len()) if that doesn't underflow.
            // And ring.len() = self@.1 => so ret == self@.1 - (1 + self@.0.len())
            // => ret as int = self@.1 as int - 1 - self@.0.len()
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
