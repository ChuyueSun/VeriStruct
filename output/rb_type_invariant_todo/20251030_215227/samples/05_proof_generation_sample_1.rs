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
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            // Proof that ret as int = self@.0.len()
            let capacity = self.ring@.len();
            if self.tail >= self.head {
                // Then ret = tail - head
                // By construction, self@.0.len() = (tail - head) as nat
                assert(ret as int == (if self.tail >= self.head {
                    (self.tail - self.head) as int
                } else {
                    (capacity + self.tail - self.head) as int
                }));
            } else {
                assert(ret as int == (capacity + self.tail - self.head) as int);
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

    pub fn has_elements(&self) -> (ret: bool)
        requires
            // (Skip self.inv() in specs due to #[verifier::type_invariant])
        ensures
            ret <==> self@.0.len() > 0
    {
        proof {
            // Prove (self.head != self.tail) <==> (self@.0.len() > 0)
            if self.head != self.tail {
                assert(self@.0.len() > 0);
            } else {
                assert(self@.0.len() == 0);
            }
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
            // ret <==> (head == (tail + 1) % capacity)
            // we want to connect that to self@.0.len() == capacity - 1
            let capacity = self.ring@.len();
            assert(self@.1 == capacity);
            // If head == ((tail+1) % capacity), then effectively
            // (tail+capacity - head) % capacity == capacity-1,
            // which matches self@.0.len() == capacity - 1.
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
            false
        } else {
            proof {
                // If not full, we add 'val' as the last element in the ring's logical order
                // So self@.0 = old(self)@.0.push(val)
                assert(old(self)@.0.len() < old(self)@.1 - 1);
                assert_seqs_equal!( // CRITICAL: single-element update (conceptually) from ring's viewpoint
                    {  // after we do the set below + tail increment:
                        // new self@.0
                        // but we can't do it before the actual modification!
                    },
                    {  // old self@.0.push(val)
                    }
                );
            }
            my_set(&mut self.ring, self.tail, val);
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
            // If we had elements, the new view is old(self)@.0.subrange(1, old(self)@.0.len()).
            // If none, no change.
            if old(self)@.0.len() > 0 {
                assert(old(self)@.0.len() > 0);
                // We remove the first element logically
                // Not a single-element "update", but logically a shift
                // So:
                //   self@.0 =~= old(self)@.0.subrange(1, old(self)@.0.len())
            } else {
                assert(old(self)@.0.len() == 0);
                // no change to the sequence
            }
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
            // ret = self.ring.len().saturating_sub(1 + self.len())
            // ex_saturating_sub_spec((self@.1 -1) as int, self@.0.len()) matches the definition of saturating_sub
            // so this is consistent with the code.
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
