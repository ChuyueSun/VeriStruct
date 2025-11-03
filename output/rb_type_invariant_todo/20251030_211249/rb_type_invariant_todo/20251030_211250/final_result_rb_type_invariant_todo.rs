/// Top-level doc comment stays here

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
            let queue_seq = Seq::new(( length ) as nat, |i: int| {
                ring_seq[((self.head as int + i) % ( ring_seq.len() ) as int) as int]
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
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn len(&self) -> (ret: usize)
        ensures
            ret == self@.0.len()
    {
        proof {
            use_type_invariant(&*self);
            assert(
                self@.0.len()
                ==
                if self.tail > self.head {
                    (self.tail - self.head) as int
                } else if self.tail < self.head {
                    (self.ring.len() - self.head + self.tail) as int
                } else {
                    0
                }
            );
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
        ensures
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&*self);
            assert((self.head != self.tail) == (self@.0.len() > 0));
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&*self);
            let ring_len = self.ring.len() as int;
            reveal(Self::view);
            if self.head == ((self.tail + 1) % ring_len) {
                if self.tail >= self.head {
                    assert(self.tail + 1 == self.head + ring_len);
                    assert(self.tail - self.head == ring_len - 1);
                } else {
                    assert(self.tail + ring_len - self.head == ring_len - 1);
                }
                assert(self@.1 == ring_len);
                assert(self@.0.len() == ring_len - 1);
            } else {
                if self@.0.len() == ring_len - 1 {
                    assert(false);
                }
            }
            assert(
                (self.head == ((self.tail + 1) % ( self.ring.len() ) as int))
                == (self@.0.len() == self@.1 - 1)
            );
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

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

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            succ <==> old(self)@.0.len() < old(self)@.1 - 1,
            succ ==> self@.0 == old(self)@.0.push(val),
            !succ ==> self@.0 == old(self)@.0
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(self.tail < old(self).ring.len()); // Added by AI
            }
            my_set(&mut self.ring, self.tail, val);

            proof {
                use_type_invariant(&*self);
                assert(old(self)@.0.len() < old(self)@.1 - 1);
                assert(self@.0 == old(self)@.0.push(val));
            }
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            ret.is_some() <==> old(self)@.0.len() > 0,
            ret.is_some() ==> (
                self@.0 =~= old(self)@.0.subrange(1, (old(self)@.0.len()) as int)
                && ret.get_Some_0() == old(self)@.0[0]
            ),
            ret.is_none() ==> self@.0 =~= old(self)@.0
    {
        if self.has_elements() {
            let val = self.ring[self.head];
            // possible division by zero
            self.head = (self.head + 1) % self.ring.len();

            proof {
                use_type_invariant(&*self);
                let old_len = old(self)@.0.len();
                assert(old_len > 0);
                assert(self@.0 =~= old(self)@.0.subrange(1, ( old_len ) as int));
                assert(val == old(self)@.0[0]);
            }
            Some(val)
        } else {
            None
        }
    }

    pub fn available_len(&self) -> (ret: usize)
        ensures
            ret == (self@.1 - 1 - self@.0.len())
    {
        proof {
            use_type_invariant(&*self);
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

    proof {
        assert(buf@.1 == len + 1);
    }

    for i in 0..len
    invariant
        buf@.0.len() == i,
        buf@.1 == len + 1,
        0 <= buf.head,
        buf.head < usize::MAX,
        1 <= buf.ring.len(),
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

// Final VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
