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
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            let n = self.ring@.len();
            let h = self.head as int;
            let t = self.tail as int;
            let cont = if (t >= h) {
                self.ring@.subrange(h, t)
            } else {
                self.ring@.subrange(h, n) + self.ring@.subrange(0, t)
            };
            (cont, (n - 1) as nat)
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
        (self.ring.len() >= 1)
        && (self.head < self.ring.len())
        && (self.tail < self.ring.len())
    }


    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        ensures
            ret == self.view().0.len(),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // By definition of self.view(),
            // if tail >= head, length = tail - head,
            // else if tail < head, length = (ring.len() - head) + self.tail,
            // else length is 0. The code below matches that exactly.
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
            ret == (self.view().0.len() > 0),
    {
        proof {
            use_type_invariant(&*self);
            // ret is true iff head != tail => that exactly means self.view().0.len() > 0.
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self.view().0.len() == self.view().1),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // A ring buffer is full iff head == (tail+1) % ring.len()
            // That corresponds to having exactly ring.len() - 1 elements,
            // i.e., self.view().0.len() == self.view().1.
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() >= 1,
        ensures
            ret.view().0.len() == 0,
            ret.view().1 == (ring.len() - 1) as nat,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }


    /// This method attempts to add a new element to the back of the ring buffer.
    ///
    /// The success of this operation is directly determined by the buffer's capacity:
    /// - If the buffer is **not full**, the element is added and the method returns `true`
    /// - If the buffer is **full**, no element is added and the method returns `false`
    ///
    /// # Arguments
    /// * `val` - The value to add to the buffer
    ///
    /// # Returns
    /// * `true` - The element was successfully added (buffer was not full)
    /// * `false` - The element could not be added (buffer was full)
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            if old(self).view().0.len() == old(self).view().1 {
                succ == false
                && self.view() =~= old(self).view()
            } else {
                succ == true
                && self.view().0 =~= old(self).view().0.push(val)
                && self.view().1 == old(self).view().1
            },
    {
        if self.is_full() {
            false
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                // We overwrite self.ring[tail] and do tail=(tail+1)%ring.len().
                // That corresponds to adding one more element in the subrange.
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer.
    ///
    /// The success of this operation is directly determined by the buffer's contents:
    /// - If the buffer is **not empty**, the front element is removed and returned as `Some(T)`
    /// - If the buffer is **empty**, no element is removed and the method returns `None`
    ///
    /// # Returns
    /// * `Some(T)` - The front element if the buffer was not empty
    /// * `None` - If the buffer was empty
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            if old(self).view().0.len() > 0 {
                matches!(ret, Option::Some(_))
                && ret.get_Some_0() == old(self).view().0[0]
                && self.view().0 =~= old(self).view().0.subrange(1, old(self).view().0.len() as int)
                && self.view().1 == old(self).view().1
            } else {
                ret == None::<T>
                && self.view() =~= old(self).view()
            },
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // If not empty => we remove the front element by incrementing head by 1 mod ring.len().
            // That shrinks the subrange by 1 at the front.
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
            ret == self.view().1 - self.view().0.len(),
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // self.view().1 = ring.len() - 1,
            // so ret = (ring.len() - 1) - self.view().0.len().
            // The code does saturating_sub(1 + self.len()), which is (ring.len() - 1 - self.len()).
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
