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
        let content = Seq::new((occupant_count) as nat, |i: int| {
            let idx = (self.head as int + i) % (capacity) as int;
            self.ring@[idx as int]
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
        vec@.len() == old(vec).len()
        no_unwind
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    /// Invariant for the ring buffer.
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

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            // We show that self@.0.len() matches the occupant count:
            assert(self.tail > self.head ==> self@.0.len() == (self.tail - self.head) as int);
            assert(self.tail < self.head ==> self@.0.len() == ((self.ring.len() - self.head) + self.tail) as int);
            assert(self.tail == self.head ==> self@.0.len() == 0);
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
        ensures
            ret == (self@.0.len() > 0)
    {
        proof {
            // If head != tail, occupant_count > 0. If head == tail, occupant_count == 0.
            assert(self.tail != self.head ==> self@.0.len() > 0);
            assert(self.tail == self.head ==> self@.0.len() == 0);
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            // The buffer is full if occupant_count == capacity - 1:
            // occupant_count == (self@.0.len()), capacity == self@.1
            // self.head == (self.tail + 1) % capacity  => occupant_count == capacity - 1
            assert({
                let capacity = self@.1;
                let occupant_count = self@.0.len();
                (self.head == ((self.tail + 1) % ( self.ring.len() ) as int)) ==> occupant_count == capacity - 1
            });
            assert({
                let capacity = self@.1;
                let occupant_count = self@.0.len();
                (self.head != ((self.tail + 1) % ( self.ring.len() ) as int)) ==> occupant_count != capacity - 1
            });
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
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

    /// This method attempts to add a new element to the back of the ring buffer.
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
                // occupant_count changes from old(self)@.0.len() to old(self)@.0.len() + 1
                assert(self@.1 == old(self)@.1);
                assert(self@.0.len() == old(self)@.0.len() + 1);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            match ret {
                Some(_) => self@.0.len() == old(self)@.0.len() - 1,
                None    => self@.0.len() == old(self)@.0.len(),
            }
    {
        proof {
            // occupant_count decreases by 1 if not empty, else remains the same
            assert(self@.1 == old(self)@.1);
            assert(self.head != self.tail ==> self@.0.len() == old(self)@.0.len() - 1);
            assert(self.head == self.tail ==> self@.0.len() == old(self)@.0.len());
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
        ensures
            (ret as int) == self@.1 - 1 - self@.0.len()
    {
        proof {
            // occupant_count = self@.0.len(), capacity = self@.1
            // available_len = capacity - 1 - occupant_count
            let occupant_count = self@.0.len();
            let cap = self@.1;
            assert((cap - 1 - occupant_count) == (self.ring.len() as int - 1 - occupant_count));
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


// VEval Score: Compilation Error: False, Verified: 3, Errors: 7, Verus Errors: 19
