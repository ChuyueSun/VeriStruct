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
    type V = Seq<T>;

    closed spec fn view(&self) -> Self::V {
        if self.tail >= self.head {
            self.ring@.subrange(self.head as int, self.tail as int)
        } else {
            self.ring@.subrange(self.head as int, self.ring@.len() as int) +
                self.ring@.subrange(0, self.tail as int)
        }
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
            // Show that the code's return value matches self@.len()
            // Case 1: tail >= head
            if self.tail >= self.head {
                // subrange(self.head, self.tail) => length = tail - head
                assert((self.tail - self.head) as int == self@.len());
            } else if self.tail < self.head {
                // subrange(self.head, ring.len()) + subrange(0, tail)
                // total length = (ring.len() - head) + tail
                assert(((self.ring.len() - self.head) + self.tail) as int == self@.len());
            } else {
                // tail == head => self@.len() = 0
                assert(0 == self@.len());
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
            self.inv()
        ensures
            self.inv(),
            ret == (self@.len() > 0)
    {
        proof {
            // self@.len() > 0  <==>  head != tail
            // If they differ, subrange is non-empty
            // If they match, subrange is empty
            if self.head != self.tail {
                assert(self@.len() > 0);
            } else {
                assert(self@.len() == 0);
            }
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
            // If is_full = (tail+1)%ring.len() == head
            // => the number of items in the buffer = ring.len()-1
            // Otherwise, < ring.len()-1
            let buffer_len = self@.len();
            let capacity_minus_one = self.ring@.len() - 1;
            if (self.head == ((self.tail + 1) % self.ring.len())) {
                assert(buffer_len == capacity_minus_one);
            } else {
                assert(buffer_len < capacity_minus_one || buffer_len <= capacity_minus_one);
            }
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
                // Because not full => we can increment length by 1
                let old_len = old(self)@.len();
                assert(old_len < self.ring@.len() - 1);
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
            // If we have elements => length is > 0; removing 1 => old_len -1
            // If none => old_len stays the same
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
            // By definition, available_len = capacity_minus_one - current_len
            let capacity_minus_one = self.ring@.len() - 1;
            let current_len = self@.len();
            assert(ret == capacity_minus_one - current_len);
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

pub fn main() {
}

}
