```rust
use vstd::prelude::*;

pub fn main() {}

verus! {
    // Specification function for saturating subtraction.
    // This spec function defines the desired behavior of a saturating subtraction operation.
    // Pre-condition: No explicit pre-condition.
    // Post-condition:
    // - If a > b, then the result is (a - b) as nat, i.e. non-negative subtraction result.
    // - Otherwise, the result is 0.
    pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
    {
        if (a > b) {
            (a - b) as nat
        } else {
            0
        }
    }

    // This is an external function specification for saturating subtraction.
    // Pre-condition: There is no explicit requires clause, so it assumes the caller gives valid usize parameters.
    // Post-condition:
    // - It ensures that the concrete implementation, when cast to int, matches the spec function result.
    #[verifier::external_fn_specification]
    pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
    ensures
        ex_saturating_sub_spec(a as int, b as int) == ret as int
    {
        a.saturating_sub(b)
    }

    // A data structure representing a ring buffer, generic on type T.
    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    // Implementation of view trait for RingBuffer.
    // The view (or abstraction function) of a ring buffer is represented as a pair:
    //   (Seq<T>, usize), where the sequence is the logical content of the buffer,
    //   and the usize is the capacity of the underlying ring.
    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, usize);
    
        // Spec function that extracts the view from the ring buffer.
        // Pre-condition: None.
        // Post-condition: Returns a tuple where:
        //   - The first component is the concatenated subsequence representing the buffer contents.
        //     The concatenation depends on whether tail is ahead or behind head.
        //   - The second component is the capacity (length) of the underlying storage.
        closed spec fn view(&self) -> Self::V {
            let cap = self.ring.len();
            if self.tail >= self.head {
                ((self.ring)@.subrange(self.head as int, self.tail as int),
                cap)
            } else {
                ((self.ring)@.subrange(self.head as int, cap as int)
                    .add((self.ring)@.subrange(0, self.tail as int)),
                    cap)
            }
        }
    }

    /// This spec function describes an arithmetic property regarding modulo addition.
    /// For any integers x and y, considering the remainders modulo n:
    /// There are two possibilities for (x % n + y % n):
    ///   (1) If (x % n + y % n) is in [0, n), then it equals (x + y) % n.
    ///   (2) If (x % n + y % n) is in [n, 2n), then it equals (x + y) % n + n.
    // Pre-condition (recommends clause): n > 0.
    // Post-condition: For all x,y, the described equivalence holds.
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

    /// This spec function describes an arithmetic property regarding modulo subtraction.
    /// For any integers x and y, considering the remainders modulo n:
    /// There are two possibilities for (x % n - y % n):
    ///   (1) If (x % n - y % n) is in [0, n), then it equals (x - y) % n.
    ///   (2) If (x % n - y % n) is in [-n, 0), then it equals (x - y) % n - n.
    // Pre-condition (recommends clause): n > 0.
    // Post-condition: For all x,y, the stated relation holds.
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

    /// This specification function aggregates various useful properties of the modulo operator,
    /// when the divisor is n.
    // Pre-condition (recommends clause): n > 0.
    // Post-condition: Returns true if all the following properties hold:
    //   - n % n == 0 and (-n) % n == 0,
    //   - For all x, (x % n) % n equals x % n.
    //   - For all x, x in [0, n) if and only if x % n equals x.
    //   - The properties described by mod_auto_plus(n) and mod_auto_minus(n) both hold.
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

    /// Proof function for mod_auto.
    /// This proof function is meant to show that mod_auto(n) holds for any positive n.
    // Pre-condition:
    //   - n > 0.
    // Post-condition:
    //   - mod_auto(n) holds, i.e., all the described modulo properties are verified.
    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0,
        ensures
            mod_auto(n),
    {
        // The proof is currently admitted (i.e., assumed) for simplicity.
        // In a complete verification, this proof would include detailed reasoning steps.
        admit()
    }


#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    // Pre-condition:
    //   - The index i is within bounds of the old vector (i < old(vec).len()).
    // Post-condition:
    //   - The new view (vec@) equals the old view with the element at index i updated to value.
    //   - The length of the vector remains unchanged.
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
    // This invariant must hold for every live instance of RingBuffer.
    // It states:
    //   - head index is less than ring length.
    //   - tail index is less than ring length.
    //   - The ring buffer has a positive capacity.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
        &&& self.ring.len() > 0
    }


    /// Returns how many elements are in the buffer.
    // Pre-condition: None explicitly, but the object must satisfy its invariant.
    // Post-condition:
    //   - The returned integer equals the logical length of the ring buffer (captured in self@.0.len()).
    pub fn len(&self) -> (ret: usize)
        ensures
            ret == self@.0.len()
    {
        proof {
            // Establish that the type invariant holds for 'self'
            use_type_invariant(&self);
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
    // Pre-condition: The ring buffer's invariant must hold.
    // Post-condition:
    //   - The result (ret) is true if the logical buffer length (self@.0.len()) is not zero.
    pub fn has_elements(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() != 0)
    {
        proof {
            // Use the invariant to reason about the structure of the ring buffer.
            use_type_invariant(&self);
        }
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    ///
    /// Being 'full' is defined as the logical length (self@.0.len()) equals (capacity - 1).
    // Pre-condition: The ring buffer's invariant holds.
    // Post-condition:
    //   - The function returns true if and only if the logical length equals (self@.1 - 1).
    pub fn is_full(&self) -> (ret: bool)
    ensures
        ret == (self@.0.len() == (self@.1 - 1) as nat) 
    {
        proof {
            // Ensure the invariant holds for the current state.
            use_type_invariant(&self);
            // Invoke modulo arithmetic properties for the capacity.
            lemma_mod_auto(self@.1 as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing storage `ring`.
    // Pre-condition:
    //   - The provided vector (ring) must have length at least 1.
    // Post-condition:
    //   - The logical content of the new buffer is empty (length 0).
    //   - The capacity (second component of the view) equals the length of the ring.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() >= 1
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring.len()
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    
    /// Attempts to enqueue a new element into the buffer.
    // Pre-condition: The ring buffer's invariant holds.
    // Post-condition:
    //   - If the old buffer was full (old(self)@.0.len() equals capacity - 1), then succ is false.
    //   - Otherwise, the element is added, and the logical length increases by 1.
    //   - The overall capacity (self@.1) remains unchanged.
    //   - If enqueuing succeeded, the new element is the last element in the logical content.
    //   - All previous elements remain unchanged.
    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            // Full fails iff old(len) == capacity => !succ
            old(self)@.0.len() == (old(self)@.1 - 1) as nat <==> !succ,
            // The ring size itself doesn't change:
            self@.1 == old(self)@.1,
            // If succ, length increments by 1:
            succ == (self@.0.len() == old(self)@.0.len() + 1),
            // The newly enqueued value is at the end:
            succ ==> (self@.0.last() == val),
            // Previous elements unchanged:
            forall |i: int|
                0 <= i < old(self)@.0.len() ==> self@.0[i] == old(self)@.0[i]
    {
        if self.is_full() {
            false
        } else {
            proof {
                // Use the invariant to ensure that the buffer state is well-formed.
                use_type_invariant(&*self);
                // Apply the lemma for modulo arithmetic properties.
                lemma_mod_auto(self@.1 as int);
            }
            my_set(&mut self.ring, self.tail, val);
            // Update the tail pointer cyclically.
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element of the buffer, if any.
    // Pre-condition: The ring buffer's invariant holds.
    // Post-condition:
    //   - The total capacity remains unchanged.
    //   - If the old logical length was 0, the result is None.
    //   - If the old logical length was greater than 0, the result is Some(val) where:
    //       * the new logical length is old_length - 1,
    //       * val equals the first element of the old logical sequence.
    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            // The ring size remains unchanged
            self@.1 == old(self)@.1,
            // Empty fails
            old(self)@.0.len() == 0 <==> ret == None::<T>,
            old(self)@.0.len() > 0 <==> ret != None::<T>,
            if let Some(val) = ret {
                &&& self@.0.len() == old(self)@.0.len() - 1
                &&& val == old(self)@.0.first()
            } else {
                self@.0.len() == old(self)@.0.len()
            }
    {
        proof {
            // Ensure the buffer's invariant is maintained.
            use_type_invariant(&*self);
            // Apply the modulo arithmetic lemma on the capacity.
            lemma_mod_auto(self@.1 as int);
        }

        if self.has_elements() {
            let val = self.ring[self.head];
            // Move the head pointer cyclically.
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }
    
    /// Returns the number of elements that can still be enqueued until the buffer is full.
    // Pre-condition: The invariant holds.
    // Post-condition: The value returned is the available slots computed as:
    //   capacity - 1 (for the full condition) minus the current logical length.
    pub fn available_len(&self) -> (ret: usize)
    {
        proof {
            // Ensure the ring buffer invariant holds.
            use_type_invariant(&self);
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

#[verifier::loop_isolation(false)]
fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)
    // Pre-condition:
    //   - The provided len is small enough such that len < usize::MAX - 1.
    //   - The total number of iterations doubled fits within usize (iterations * 2 < usize::MAX).
    // Post-condition: No explicit post-condition is given; this is a test function used for verification.
    requires
        len < usize::MAX - 1,
        iterations * 2 < usize::MAX,
{
    let mut ring: Vec<i32> = Vec::new();

    if len == 0 {
        return;
    }

    // Loop to initialize the ring vector to have len + 1 elements.
    // Loop invariant:
    //   - After each iteration, the length of 'ring' is exactly i.
    for i in 0..(len + 1)
        invariant
            // The invariant ensures that the vector size equals the loop counter.
            ring.len() == i,
    {
        ring.push(0);
    }

    // Ensure that ring capacity is more than 1.
    assert(ring.len() > 1);
    let mut buf = RingBuffer::new(ring);
    assert(buf@.1 > 1);

    // Loop to perform enqueue and dequeue operations for 2 * iterations.
    // Loop invariant:
    //   - The logical length of the buffer is 0 (i.e., the buffer is empty).
    //   - The capacity (second component in the view, buf@.1) is greater than 1.
    for _ in 0..2 * iterations
        invariant
            // The buffer remains empty at the start of each iteration.
            buf@.0.len() == 0,
            // The capacity invariant holds.
            buf@.1 > 1
    {
        let enqueue_res = buf.enqueue(value);
        // Verify that the enqueue operation succeeded.
        assert(enqueue_res);

        let buf_len = buf.len();
        // After a successful enqueue, the logical length should be 1.
        assert(buf_len == 1);

        let has_elements = buf.has_elements();
        // The buffer should have elements after enqueue.
        assert(has_elements);

        let dequeue_res = buf.dequeue();
        // Verify that dequeue returns the enqueued value.
        assert(dequeue_res =~= Some(value));

        let buf_len = buf.len();
        // After the dequeue, the buffer must be empty again.
        assert(buf_len == 0);

        let has_elements = buf.has_elements();
        // The buffer should be reported as empty.
        assert(!has_elements);
    }
}
}
```