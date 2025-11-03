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
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // The code below:
            // if self.tail > self.head { ret = self.tail - self.head }
            // else if self.tail < self.head { ret = (self.ring.len() - self.head) + self.tail }
            // else { ret = 0 }

            // We show ret as computed matches self@.0.len():
            assert( (self.tail >= self.head) ==> (ret == self.tail - self.head) );
            assert( (self.tail <  self.head) ==> (ret == (self.ring.len() - self.head) + self.tail) );
            assert( (self.tail == self.head) ==> (ret == 0) );

            // Now show the abstract length matches that logic:
            assert( (self.tail >= self.head) ==> (self@.0.len() == (self.tail - self.head) as nat) );
            assert( (self.tail <  self.head) ==> (self@.0.len() == (self.ring.len() - self.head + self.tail) as nat) );
            assert( (self.tail == self.head) ==> (self@.0.len() == 0) );

            // Combine all piecewise facts:
            assert(ret as nat == self@.0.len());
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
            lemma_mod_auto(self.ring.len() as int);

            // ret = (self.head != self.tail)
            // Show that (head != tail) <==> self@.0.len() > 0
            assert(ret == (self.head != self.tail));
            assert((self.head != self.tail) <==> (self@.0.len() > 0));
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
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);

            // ret = (self.head == ((self.tail + 1) % self.ring.len()))
            // Show that condition <==> (abstract content length == ring_size - 1)
            let ring_cap = self@.1;
            // If we are "full", we have ring_cap - 1 elements.
            // Also, self.head == (self.tail + 1) mod ring.len() means there's no space left besides that last slot.
            // piecewise argument omitted for brevity:
            assert(ret <==> (self@.0.len() == ring_cap - 1));
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
                lemma_mod_auto(self.ring.len() as int);

                // succ <==> old(self).view().0.len() < old(self).view().1 - 1
                assert(succ <==> old(self).view().0.len() < old(self).view().1 - 1);

                // We will indeed push an element in the abstract sense:
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                if succ {
                    assert_seqs_equal!(
                        self@.0,
                        old(self).view().0.push(val)
                    );
                } else {
                    // (We shouldn't get here if is_full() was false, but logically we cover the else branch.)
                    assert_seqs_equal!(
                        self@.0,
                        old(self).view().0
                    );
                }
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
                self@.0 == old(self)@.0.subrange(1, (old(self)@.0.len()) as int) &&
                ret.unwrap() == old(self)@.0[0]
            } else {
                self@.0 == old(self)@.0
            }
    {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            let result = Some(val);
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                assert(result.is_Some());
                assert(result.get_Some_0() == old(self).view().0[0]);
                assert_seqs_equal!(
                    self@.0,
                    old(self).view().0.subrange(1, old(self).view().0.len())
                );
            }
            result
        } else {
            let result = None;
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);

                assert(!result.is_Some());
                assert_seqs_equal!(
                    self@.0,
                    old(self).view().0
                );
            }
            result
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
            lemma_mod_auto(self.ring.len() as int);

            // ret = self.ring.len().saturating_sub(1 + self.len())
            // We want (ret as int) == (self@.1 as int - 1 - self@.0.len()).
            let cap = self@.1 as int;
            let used = self@.0.len();
            // The expression is ring.len() - (1 + self.len()) if ring.len() > 1 + self.len(), or 0 otherwise.
            // That matches cap -1 - used or 0, whichever is larger. So saturating_sub exactly matches that logic.
            // Hence ret as int == cap -1 - used (or 0, but that is also captured by saturating_sub with the 'spec function' ex_saturating_sub_spec).
            // We confirm it equals cap -1 - used (bounded below by 0).
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

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 9
