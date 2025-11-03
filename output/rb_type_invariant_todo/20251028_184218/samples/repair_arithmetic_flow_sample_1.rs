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
        let capacity = self.ring@.len();
        let occupant_count = if self.tail >= self.head {
            (self.tail - self.head) as int
        } else {
            (capacity - self.head) as int + self.tail as int
        };
        let content = Seq::new(occupant_count as nat, |i: int| {
            let idx = (self.head as int + i) % (capacity) as int;
            self.ring@[idx]
        });
        (content, capacity)
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
        let capacity = self.ring.len();
        let occupant_count = if self.tail >= self.head {
            (self.tail - self.head) as int
        } else {
            (capacity - self.head) as int + self.tail as int
        };
        capacity > 0
        && self.head < capacity
        && self.tail < capacity
        && occupant_count >= 0
        && occupant_count < (capacity as int)
    }

    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            assert(self.tail >= self.head ==> (self.tail as int - self.head as int) == self@.0.len());
            assert(self.tail < self.head ==> ((self.ring.len() as int - self.head as int) + self.tail as int) == self@.0.len());
            assert(self.tail == self.head ==> self@.0.len() == 0);
        }
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            proof {
                assert(self.ring.len() >= self.head);
                assert(((self.ring.len() - self.head) + self.tail) < 2 * self.ring.len());
            }
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
            assert(self.head != self.tail ==> self@.0.len() > 0);
            assert(self.head == self.tail ==> self@.0.len() == 0);
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            let capacity = self@.1;
            let occupant_count = self@.0.len();
            assert(self.head == ((self.tail + 1) % ( self.ring.len() ) as int) ==> occupant_count == capacity - 1);
            assert(self.head != ((self.tail + 1) % ( self.ring.len() ) as int) ==> occupant_count != capacity - 1);
            assert(self.tail + 1 <= self.ring.len());
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
        ensures
            self@.1 == old(self)@.1,
            if succ {
                self@.0.len() == old(self)@.0.len() + 1
            } else {
                self@.0.len() == old(self)@.0.len()
            }
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(self@.1 == old(self)@.1);
                assert(self@.0.len() == old(self)@.0.len() + 1);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            match ret {
                Some(_) => self@.0.len() == old(self)@.0.len() - 1,
                None    => self@.0.len() == old(self)@.0.len(),
            }
    {
        proof {
            assert(self@.1 == old(self)@.1);
        }
        if self.has_elements() {
            proof {
                assert(self.inv());
                assert(self.head < self.ring.len());
            }
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    pub fn available_len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.1 - 1 - self@.0.len()
    {
        proof {
            assert(1 + self@.0.len() <= self@.1);
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

pub fn main() {
}

}

// VEval Score: Compilation Error: False, Verified: 3, Errors: 6, Verus Errors: 10
