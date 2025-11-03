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

// ================== ADDED VIEW IMPLEMENTATION ====================
impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let capacity = self.ring@.len();
        let count: nat = if self.tail >= self.head {
            (self.tail - self.head) as nat
        } else {
            (capacity + self.tail - self.head) as nat
        };
        let seq = Seq::new(count, |i: int| {
            self.ring@[((self.head as int + i) % (capacity) as int) as int]
        });
        (seq, capacity)
    }
}
// ================================================================

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
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            let capacity = self.ring@.len();
            // Show piecewise that the returned value matches self@.0.len()
            assert(self.tail > self.head) ==> {
                assert((self.tail - self.head) as int == self@.0.len());
            };
            assert(self.tail < self.head) ==> {
                assert(((capacity + self.tail) - self.head) as int == self@.0.len());
            };
            assert(self.tail == self.head) ==> {
                assert(self@.0.len() == 0);
            };
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
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            ret <==> self@.0.len() > 0
    {
        proof {
            let len0 = self@.0.len();
            // The ring buffer has elements iff head != tail
            assert(self.head != self.tail) ==> {
                assert(len0 > 0);
            };
            assert(self.head == self.tail) ==> {
                assert(len0 == 0);
            };
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        requires
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            ret <==> self@.0.len() == self@.1 - 1
    {
        proof {
            let capacity = self@.1;
            let len0 = self@.0.len();
            // If ring is full, we have head == (tail+1) mod capacity
            // which means difference = capacity - 1
            // So piecewise:
            assert(self.head == ((self.tail + 1) % self.ring.len())) ==> {
                assert(len0 == capacity - 1);
            };
            assert(len0 == capacity - 1) ==> {
                assert(self.head == ((self.tail + 1) % self.ring.len()));
            };
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0
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
        requires
            old(self)@.1 > 0, // capacity > 0
        ensures
            (succ <==> old(self)@.0.len() < old(self)@.1 - 1),
            succ ==> (self@.0 =~= old(self)@.0.push(val)),
            !succ ==> (self@.0 =~= old(self)@.0),
            self@.1 == old(self)@.1
    {
        if self.is_full() {
            proof {
                // If it's full, we can't add
                assert(!(old(self)@.0.len() < old(self)@.1 - 1));
            }
            false
        } else {
            proof {
                // We know there's space: old(self)@.0.len() < old(self)@.1 - 1
                assert(old(self)@.0.len() < old(self)@.1 - 1);
            }
            my_set(&mut self.ring, self.tail, val);
            proof {
                // We've updated one element in the ring Vec:
                // Use the macro for single-element update in a Vec
                assert_seqs_equal!(self.ring@, old(self).ring@.update(self.tail as int, val));

                // Now show bridging: we appended 'val' in the ring buffer's abstract sequence
                let old_seq = old(self)@.0;
                let new_seq = self@.0;
                let n = old_seq.len();
                // We expect new_seq to be old_seq.push(val)
                assert(new_seq.len() == n + 1);
                assert forall|i: int|
                    0 <= i && i < n ==> new_seq[i] == old_seq[i]
                by {
                    // no interior statements needed
                };
                assert(new_seq[n] == val);
            }
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            old(self)@.1 > 0
        ensures
            (ret.is_None() <==> old(self)@.0.len() == 0),
            ret.is_Some() ==> ret.get_Some_0() == old(self)@.0[0],
            ret.is_Some() ==> self@.0 =~= old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int),
            ret.is_None() ==> self@.0 =~= old(self)@.0,
            self@.1 == old(self)@.1
    {
        proof {
            // We'll bridge only if we actually remove something
            assert(self.has_elements()) ==> {
                let old_seq = old(self)@.0;
                let n = old_seq.len();
                assert(n > 0);
                let new_seq = self@.0;
                // We expect new_seq = old_seq.subrange(1, n)
                assert(new_seq.len() == n - 1);
                assert forall|i: int|
                    0 <= i && i < n - 1 ==> new_seq[i] == old_seq[i+1]
                by {
                    // no interior statements needed
                };
            };
            assert(!self.has_elements()) ==> {
                // No change
                assert(self@.0 =~= old(self)@.0);
            };
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
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            (ret as int) == ex_saturating_sub_spec((self@.1 - 1) as int, ( self@.0.len() ) as int)
    {
        proof {
            let cap_minus_1 = (self@.1 - 1) as int;
            let used = self@.0.len() as int;
            // ex_saturating_sub_spec checks if cap_minus_1 > used or not
            assert(cap_minus_1 >= used) ==> {
                assert(ex_saturating_sub_spec(cap_minus_1, used) == cap_minus_1 - used);
            };
            assert(cap_minus_1 < used) ==> {
                assert(ex_saturating_sub_spec(cap_minus_1, used) == 0);
            };
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

} // verus! block ends here

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
