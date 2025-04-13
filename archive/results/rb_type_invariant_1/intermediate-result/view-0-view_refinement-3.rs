use vstd::prelude::*;

pub fn main() {}

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

    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, nat);

        closed spec fn view(&self) -> Self::V {
            let capacity = self.ring@.len();
            let contents =
                if self.tail >= self.head {
                    self.ring@.subrange(self.head as int, self.tail as int)
                } else {
                    self.ring@.subrange(self.head as int, capacity)
                        ++ self.ring@.subrange(0, self.tail as int)
                };
            (contents, capacity)
        }
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
    closed spec fn inv(&self) -> ( bool ) as bool {
        // TODO: specify the invariant
    }

    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        // TODO: add requires and ensures
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self@.1 as int);
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
        // TODO: add requires and ensures
    {
        proof {
            use_type_invariant(&self);
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    ///
    /// Being 'full' means `self@.len() == (self.ring.len() - 1) as nat`.
    pub fn is_full(&self) -> (ret: bool)
        // TODO: add requires and ensures
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self@.1 as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        // TODO: add requires and ensures
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    
    /// If the buffer isn't full, adds a new element to the back.
    /// Returns whether the element was added.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        // TODO: add requires and ensures
    {
        if self.is_full() {
            false
        } else {
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self@.1 as int);
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element, if any.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        // TODO: add requires and ensures
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self@.1 as int);
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
        // TODO: add requires and ensures
    {
        proof {
            use_type_invariant(&self);
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

#[verifier::loop_isolation(false)]
fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)
    requires
        len < usize::MAX - 1,
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

    assert(ring.len() > 1);
    let mut buf = RingBuffer::new(ring);
    assert(buf@.1 > 1);

    for _ in 0..2 * iterations
        invariant
            buf@.0.len() == 0,
            buf@.1 > 1
    {
        let enqueue_res = buf.enqueue(value);
        assert(enqueue_res);

        let buf_len = buf.len();
        assert(buf_len == 1);

        let has_elements = buf.has_elements();
        assert(has_elements);

        let dequeue_res = buf.dequeue();
        assert(dequeue_res =~= Some(value));

        let buf_len = buf.len();
        assert(buf_len == 0);

        let has_elements = buf.has_elements();
        assert(!has_elements);
    }
}
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: expected an expression
// {"$message_type":"diagnostic","message":"expected an expression","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuqt8fr6d","byte_start":994,"byte_end":995,"line_start":39,"line_end":39,"column_start":26,"column_end":27,"is_primary":true,"text":[{"text":"                        ++ self.ring@.subrange(0, self.tail as int)","highlight_start":26,"highlight_end":27}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected an expression\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuqt8fr6d:39:26\n   |\n39 |                         ++ self.ring@.subrange(0, self.tail as int)\n   |                          ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 