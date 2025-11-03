use vstd::prelude::*;
use vstd::assert_seqs_equal;

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
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let ring_view = self.ring@;
        let c = if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.ring.len() - self.head + self.tail
        };
        let content = Seq::new(c as nat, |i: int|
            ring_view[((self.head as int + i) % ( ring_view.len() ) as int) as int]
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
        vec@.len() == old(vec).len()
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
            let ring_view = self.ring@;
            // Bridge from ring indices to abstract content length:
            assert(self.tail > self.head ==> self@.0.len() == (self.tail - self.head) as nat);
            assert(self.tail < self.head ==> self@.0.len() == ((ring_view.len() - self.head + self.tail) as nat));
            assert(self.tail == self.head ==> self@.0.len() == 0);

            // Match the code's return value to that same logic:
            assert(self.tail > self.head ==> ret == self.tail - self.head);
            assert(self.tail < self.head ==> ret == ring_view.len() - self.head + self.tail);
            assert(self.tail == self.head ==> ret == 0);

            // Conclude ret as nat matches the content length
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
            // ret is computed as (self.head != self.tail)
            // Bridge to the content:
            assert(self@.0.len() > 0 ==> self.head != self.tail);
            assert(self@.0.len() == 0 ==> self.head == self.tail);
            assert(ret == (self.head != self.tail));
            assert(ret == (self@.0.len() > 0));
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
            let ring_view = self.ring@;
            // ret = (self.head == ((self.tail + 1) % ring.len()))
            // Combined with the definition of self@.0.len(), we get:
            // ret <==> self@.0.len() == ring_view.len() as nat - 1
            assert(ret ==> self@.0.len() == ring_view.len() as nat - 1);
            assert(!ret ==> self@.0.len() < ring_view.len() as nat - 1);
            assert(ret == (self@.0.len() == (self@.1 - 1)));
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
                // We know from the ensures: old(self)@.0.len() < old(self)@.1 - 1
                // The ring is about to be updated at old(self).tail:
                assert(old(self)@.0.len() < old(self)@.1 - 1);

                // After the actual modification:
                assert_seqs_equal!(self.ring@, old(self).ring@.update(old(self).tail as int, val));

                // Now, tail is advanced by 1 => new content is old content push val
                assert(self@.0 == old(self)@.0.push(val));
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
            ret.is_some() <==> old(self)@.0.len() > 0,
            if ret.is_some() {
                self@.0 == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int) &&
                ret.unwrap() == old(self)@.0[0]
            } else {
                self@.0 == old(self)@.0
            }
    {
        proof {
            // The code checks has_elements -> Some, else None
            // So we show the bridging with implies:
            assert((old(self)@.0.len() > 0) ==> ret.is_some());
            assert((old(self)@.0.len() == 0) ==> ret.is_none());
            // If it was non-empty:
            assert((old(self)@.0.len() > 0) ==> self@.0 == old(self)@.0.subrange(1, old(self)@.0.len()));
            assert((old(self)@.0.len() > 0) ==> ret.unwrap() == old(self)@.0[0]);
            // If it was empty:
            assert((old(self)@.0.len() == 0) ==> self@.0 == old(self)@.0);
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
            (ret as int) == self@.1 as int - 1 - self@.0.len()
    {
        proof {
            let ring_view = self.ring@;
            let c = self@.0.len(); // current content length
            let r = ring_view.len(); // ring capacity
            // ret = r.saturating_sub(1 + self.len())
            // We want ret as int = r - 1 - c
            // Because self@.0.len() == c and self@.1 == r
            // saturating_sub could clamp to 0, but we know ring > 0 and content <= r-1
            assert((r as int) - 1 - c as int >= 0);
            assert((ret as int) == (r as int).saturating_sub(1 + c as int));
            assert(ret as int == r as int - 1 - c as int);
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 14
