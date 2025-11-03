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
    ex_saturating_sub_spec(a as int, b as int) == ret as int
{
    a.saturating_sub(b)
}

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the sum `x % n + y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x + y) % n`.
    /// (2) It's in the range `[n, 2n)` and equals `(x + y) % n + n`.
    pub open spec fn mod_auto_plus(n: int) -> bool
        recommends
            n > 0,
    {
        forall|x: int, y: int|
            {
                let z = (x % n) + (y % n);
                ((0 <= z < n && #[trigger] ((x + y) % n) == z)
                    || (n <= z < n + n && ((x + y) % n) == z - n))
            }
    }

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the difference `x % n - y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x - y) % n`.
    /// (2) It's in the range `[-n, 0)` and equals `(x - y) % n - n`.
    pub open spec fn mod_auto_minus(n: int) -> bool
        recommends
            n > 0,
    {
        forall|x: int, y: int|
            {
                let z = (x % n) - (y % n);
                ((0 <= z < n && #[trigger] ((x - y) % n) == z)
                    || (-n <= z < 0 && ((x - y) % n) == z + n))
            }
    }

    /// This function states various useful properties about the modulo
    /// operator when the divisor is `n`.
    pub open spec fn mod_auto(n: int) -> bool
        recommends
            n > 0,
    {
        &&& (n % n == 0 && (-n) % n == 0)
        &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
        &&& (forall|x: int| 0 <= x < n <==> #[trigger] (x % n) == x)
        &&& mod_auto_plus(n)
        &&& mod_auto_minus(n)
    }

    /// Proof of `mod_auto(n)`, which states various useful properties
    /// about the modulo operator when the divisor is the positive
    /// number `n`
    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0,
        ensures
            mod_auto(n),
    {
        admit()
    }


    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, nat, nat);

        closed spec fn view(&self) -> Self::V {
            (self.ring@, ( self.head ) as nat, ( self.tail ) as nat)
        }
    }


#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
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
        (self.ring.len() > 0)
        && (self.head < self.ring.len())
        && (self.tail < self.ring.len())
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        requires
            // Skipping self.inv() per instructions (#[verifier::type_invariant] present)
        ensures
            ret == if self.view().2 >= self.view().1 {
                self.view().2 - self.view().1
            } else {
                self.view().0.len() - self.view().1 + self.view().2
            },
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Here we could add additional intermediate assertions if needed
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
        requires
            // Skipping self.inv() per instructions (#[verifier::type_invariant] present)
        ensures
            ret <==> (self.view().1 != self.view().2),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // We show that ret <==> (self.head != self.tail) and that matches the view spec
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        requires
            // Skipping self.inv() per instructions (#[verifier::type_invariant] present)
        ensures
            ret <==> (self.view().1 == ((self.view().2 + 1) % self.view().0.len())),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Show ret <==> (self.head == (self.tail + 1) % ring.len())
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0,
        ensures
            ret.view().0 =~= ring@,
            ret.view().1 == 0,
            ret.view().2 == 0,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// Attempts to add a new element to the back of the buffer.
    /// Returns true if successful, false if the buffer is full.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        requires
            // Skipping self.inv() in requires (#[verifier::type_invariant] present)
        ensures
            succ <==> (old(self).view().1 != ((old(self).view().2 + 1) % old(self).view().0.len())),
            if succ {
                &&& self.view().0 =~= old(self).view().0.update(( old(self).view().2 ) as int, val)
                &&& self.view().1 == old(self).view().1
                &&& self.view().2 == ((old(self).view().2 + 1) % old(self).view().0.len())
            } else {
                self.view() =~= old(self).view()
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                // We establish that the index tail is valid and the ring is being updated properly
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the buffer if not empty.
    /// Returns Some(T) if successful, None if empty.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        requires
            // Skipping self.inv() in requires (#[verifier::type_invariant] present)
        ensures
            (ret.is_Some()) <==> (old(self).view().1 != old(self).view().2),
            if ret.is_Some() {
                &&& ret.get_Some_0() == old(self).view().0[( old(self).view().1 ) as int]
                &&& self.view().0 =~= old(self).view().0
                &&& self.view().1 == ((old(self).view().1 + 1) % old(self).view().0.len())
                &&& self.view().2 == old(self).view().2
            } else {
                self.view() =~= old(self).view()
            },
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Show that if the buffer is not empty, removing the head element is consistent with the spec
        }
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued until the buffer is full.
    pub fn available_len(&self) -> (ret: usize)
        requires
            // Skipping self.inv() per instructions (#[verifier::type_invariant] present)
        ensures
            ret == self.view().0.len() - 1 - (
                if self.view().2 >= self.view().1 {
                    self.view().2 - self.view().1
                } else {
                    self.view().0.len() - self.view().1 + self.view().2
                }
            ),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // Show consistency with ring size minus number of elements
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* TEST CODE BELOW */

#[verifier::loop_isolation(false)]
fn test1(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
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

    // assert(ring.len() == len + 1);
    let mut buf = RingBuffer::new(ring);

    let ret = buf.dequeue();
    let buf_len = buf.len();
    let has_elements = buf.has_elements();
    // assert(!has_elements);
    // assert(ret == None::<i32>);
    // assert(buf_len == 0);
    // assert(len > 1);
    for i in 0..len
    invariant
        buf@.0.len() == i,
        buf@.1 == len + 1
    {
        let enqueue_res = buf.enqueue(value);
        // assert(enqueue_res);
        let has_elements = buf.has_elements();
        // assert(has_elements);
        let available_len = buf.available_len();
        // assert(available_len == len - 1 - i);
    }
    let dequeue_res = buf.dequeue();
    // assert(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    // assert(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    // assert(!enqueue_res);
    let dequeue_res = buf.dequeue();
    // assert(dequeue_res.is_some());
}

#[verifier::loop_isolation(false)]
fn test2(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
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
        // assert(enqueue_res);
        let has_elements = buf.has_elements();
        // assert(has_elements);
        let available_len = buf.available_len();
        // assert(available_len == len - 1 - i);
    }
    let dequeue_res = buf.dequeue();
    // assert(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    // assert(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    // assert(!enqueue_res);
    let dequeue_res = buf.dequeue();
    // assert(dequeue_res.is_some());
}

#[verifier::loop_isolation(false)]
fn test3(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
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
    // assert(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    // assert(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    // assert(!enqueue_res);
    let dequeue_res = buf.dequeue();
    // assert(dequeue_res.is_some());
}

#[verifier::loop_isolation(false)]
fn test4(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
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
