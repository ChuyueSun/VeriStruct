use vstd::prelude::*;

// Main entry point of the program. No operational code here.
pub fn main() {}

verus! {
    // Specification function for saturating subtraction.
    // This function defines the mathematical behavior of saturating subtraction.
    // Precondition: None (it is a spec function with no side effects).
    // Postcondition: Returns (a - b) as a natural number if a > b, otherwise returns 0.
    pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
    {
        if (a > b) {
            (a - b) as nat
        } else {
            0
        }
    }

    // External function specification for saturating subtraction.
    // This executable function wraps the Rust saturating_sub method.
    // Precondition: None explicitly stated.
    // Postcondition: The returned result, when converted to int, equals the result given
    // by the spec function ex_saturating_sub_spec (after converting the arguments to int).
    #[verifier::external_fn_specification]
    pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
    ensures
        // Ensures that the computed result matches the specification defined by ex_saturating_sub_spec.
        ex_saturating_sub_spec(a as int, b as int) == ret as int
    {
        a.saturating_sub(b)
    }

    // Data structure representing a Ring Buffer.
    // The ring buffer is parameterized by type T which must be Copy.
    // It contains a backing vector along with head and tail indices.
    pub struct RingBuffer<T: Copy> {
        ring: Vec<T>,
        head: usize,
        tail: usize,
    }

    // Implementation of the View trait for RingBuffer.
    // The view returns a tuple containing:
    //   - A sequence (Seq<T>) representing the logically stored elements in order.
    //   - The capacity of the underlying vector (usize).
    impl<T: Copy> View for RingBuffer<T> {
        type V = (Seq<T>, usize);
    
        // Spec function to observe the current logical contents and capacity of the RingBuffer.
        // It returns the buffer elements by reading elements in the ring based on head and tail pointers.
        // Precondition: None (spec functions are side-effect free).
        // Postcondition: Returns a tuple where the first element is the logical sequence of elements
        // from self.ring between head and tail (handling wrap-around), and the second element is the capacity.
        closed spec fn view(&self) -> Self::V {
            let cap = self.ring.len();
            if self.tail >= self.head {
                // When the tail is ahead of the head, the view is simply the subrange from head to tail.
                ((self.ring)@.subrange(self.head as int, self.tail as int),
                cap)
            } else {
                // When the tail has wrapped around, the view is the concatenation of the subrange from head to cap
                // and the subrange from 0 to tail.
                ((self.ring)@.subrange(self.head as int, cap as int)
                    .add((self.ring)@.subrange(0, self.tail as int)),
                    cap)
            }
        }
    }

    // Specification function for modulo addition properties.
    // It asserts that for any two integers x and y: their modular remainders added together
    // fall into one of two specific cases regarding the modulo operation.
    // Precondition (recommends): n > 0 (divisor should be positive).
    // Postcondition: For all x and y, the sum of (x % n) and (y % n) is either in the range [0, n)
    // and equals (x+y) % n, or in the range [n, 2n) and equals (x+y) % n plus n.
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

    // Specification function for modulo subtraction properties.
    // It asserts that for any two integers x and y: their modular differences
    // fall into one of two cases regarding the modulo operation.
    // Precondition (recommends): n > 0 (divisor should be positive).
    // Postcondition: For all x and y, the difference (x % n) - (y % n) is either in the range [0, n)
    // and equals (x - y) % n, or in the range [-n, 0) and equals (x - y) % n minus n.
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

    // Combined specification function detailing various useful modulo properties when the divisor is n.
    // Precondition (recommends): n > 0.
    // Postcondition: Asserts that the divisor divides itself and its negation cleanly,
    // that taking the modulo twice is idempotent, that numbers in [0, n) remain unchanged by modulo n,
    // and also satisfies the properties defined in mod_auto_plus and mod_auto_minus.
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

    // Proof function for mod_auto(n).
    // Precondition: n must be greater than 0.
    // Postcondition: Proves that all the properties specified in mod_auto(n) hold.
    // Note: The body currently contains 'admit()', meaning that the proof is assumed rather than constructed.
    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0,
        ensures
            mod_auto(n),
    {
        admit()
    }


    // External body function to update an element in a vector at a given index.
    // Precondition: Requires that 'i' is a valid index within the original vector (i < old(vec).len()).
    // Postcondition:
    //   - The ghost/abstract view of 'vec' is equal to old(vec) updated at index i with 'value'.
    //   - The length of the ghost view of the vector remains unchanged.
    //   - The function does not cause unwinding.
    #[verifier::external_body]
    fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
        requires
            // Precondition: i must be a valid index in the vector.
            i < old(vec).len(),
        ensures
            // Postcondition: The ghost view of the vector is updated at position i.
            vec@ == old(vec)@.update(i as int, value),
            // Postcondition: The length of the vector remains unchanged.
            vec@.len() == old(vec).len()
            no_unwind
    {
        vec[i] = value;
    }


    impl<T: Copy> RingBuffer<T> {
        /// Invariant for the ring buffer.
        // This type invariant ensures that:
        //   - The head index is always within the bounds of the vector.
        //   - The tail index is always within the bounds of the vector.
        //   - The underlying vector (ring) is non-empty.
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            &&& self.head < self.ring.len()
            &&& self.tail < self.ring.len()
            &&& self.ring.len() > 0
        }

        /// Returns how many elements are in the buffer.
        // Precondition: None explicitly, but the type invariant must hold.
        // Postcondition: The returned value equals the logical length of the buffer, as captured
        // by the ghost view (i.e., self@.0.len()).
        pub fn len(&self) -> (ret: usize)
            ensures
                ret == self@.0.len()
        {
            proof {
                // Use the type invariant to ensure that indices are valid.
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
        // Precondition: The type invariant holds.
        // Postcondition: Returns true if and only if the logical view of the ring buffer (self@.0)
        // is non-empty.
        pub fn has_elements(&self) -> (ret: bool)
            ensures
                ret == (self@.0.len() != 0)
        {
            proof {
                // Assert that the ring buffer invariants hold.
                use_type_invariant(&self);
            }
            // The buffer has elements if head and tail are not equal.
            self.head != self.tail
        }

        /// Returns true if the buffer is full, false otherwise.
        /// Being 'full' means that the number of elements in the buffer equals capacity - 1.
        // Precondition: The type invariant holds.
        // Postcondition: Returns true if and only if the ghost view length equals (capacity - 1).
        pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == (self@.1 - 1) as nat) 
        {
            proof {
                // Ensure the ring buffer invariant holds and the modulo properties for capacity.
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            // The buffer is full if advancing tail by one (mod capacity) would equal head.
            self.head == ((self.tail + 1) % self.ring.len())
        }

        /// Creates a new RingBuffer with the given backing 'ring' storage.
        // Precondition: The given vector 'ring' must have at least one element (ring.len() >= 1).
        // Postcondition:
        //   - The logical view of the ring buffer is initially empty.
        //   - The capacity (ghost view) equals the length of the provided vector.
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

        pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            self@.1 == old(self)@.1,
            ((old(self)@.1 - 1) as nat != old(self)@.0.len() ==>
                succ &&
                self@.0 =~= old(self)@.0.add(seq![val])) &&
            ((old(self)@.1 - 1) as nat == old(self)@.0.len() ==>
                (succ == false && self@ =~= old(self)@))
        {
            if self.is_full() {
                false
            } else {
                proof {
                    // Use the type invariant and modulo arithmetic properties to validate the upcoming update.
                    use_type_invariant(&*self);
                    lemma_mod_auto(self@.1 as int);
                }
                // Set the current tail position to the new value.
                my_set(&mut self.ring, self.tail, val);
                // Advance the tail pointer with wrap-around.
                self.tail = (self.tail + 1) % self.ring.len();
                true
            }
        }
        pub fn dequeue(&mut self) -> (ret: Option<T>)
        // Precondition: No explicit precondition is required since the type invariant ensures valid indices.
        // Postcondition:
        //   - If the buffer is empty (ret == None), then the logical view remains unchanged.
        //   - Otherwise, if an element is removed (ret == Some(x)), then:
        //       * x is equal to the first element of the old logical view.
        //       * The new logical view is the old logical view with its first element removed.
        //       * The capacity remains unchanged.
        ensures
            self@.1 == old(self)@.1,
            (old(self)@.0.len() != 0 ==>
                ret =~= Some(old(self)@.0.index(0)) &&
                self@.0 =~= old(self)@.0.skip(1)) &&
            (old(self)@.0.len() == 0 ==>
                (ret == None::<T> && self@ =~= old(self)@))

        {
            proof {
                // Validate that the ring buffer invariants and modulo properties hold.
                use_type_invariant(&*self);
                lemma_mod_auto(self@.1 as int);
            }

            if self.has_elements() {
                // If there are elements, remove the element at the head.
                let val = self.ring[self.head];
                // Advance the head pointer with wrap-around.
                self.head = (self.head + 1) % self.ring.len();
                Some(val)
            } else {
                None
            }
        }
    
        /// Returns the number of elements that can still be enqueued until the buffer is full.
        // Precondition: The type invariant holds.
        // Postcondition: The returned value equals the available capacity,
        // computed as capacity minus the current logical length minus one (reserve slot for full condition).
        pub fn available_len(&self) -> (ret: usize)
        ensures ret == self@.1 - self@.0.len() - 1
        {
            proof {
                // Ensure that the ring buffer invariants hold before performing arithmetic.
                use_type_invariant(&self);
            }
            // Compute available space by subtracting one (for disambiguation of full vs. empty) from capacity.
            self.ring.len().saturating_sub(1 + self.len())
        }
    }

    // Test function for the enqueue and dequeue operations on RingBuffer.
    // This function operates generically over the given parameters.
    // Precondition:
    //   - The given length 'len' must be less than usize::MAX - 1 to avoid overflow.
    //   - The number of iterations (times 2) must also be less than usize::MAX to avoid arithmetic overflow.
    #[verifier::loop_isolation(false)]
    fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)
        requires
            len < usize::MAX - 1,
            iterations * 2 < usize::MAX,
    {
        let mut ring: Vec<i32> = Vec::new();

        // If len is 0, there's nothing to test, so return early.
        if len == 0 {
            return;
        }

        // Loop to populate the vector 'ring' with (len+1) zeros.
        // Loop Invariant:
        //   - At each iteration 'i', the length of 'ring' equals i.
        for i in 0..(len + 1)
            invariant
                // Invariant: The vector's length is exactly equal to the current loop index 'i'.
                ring.len() == i,
        {
            ring.push(0);
        }

        assert(ring.len() > 1);
        let mut buf = RingBuffer::new(ring);
        assert(buf@.1 > 1);

        // Loop to perform enqueue and dequeue operations repeatedly.
        // Loop Invariants:
        //   - After each iteration, the logical view of the buffer is empty (buf@.0.len() == 0).
        //   - The capacity (buf@.1) remains greater than 1.
        for _ in 0..2 * iterations
            invariant
                // Invariant: At the start of each iteration, the ring buffer's logical length is 0.
                buf@.0.len() == 0,
                // Invariant: The capacity of the ring buffer is always greater than 1.
                buf@.1 > 1
        {
            let enqueue_res = buf.enqueue(value);
            // Assert that enqueue was successful.
            assert(enqueue_res);

            let buf_len = buf.len();
            // After enqueue, the buffer should have exactly one element.
            assert(buf_len == 1);

            let has_elements = buf.has_elements();
            // The buffer should report containing elements.
            assert(has_elements);

            let dequeue_res = buf.dequeue();
            // The dequeued result should be Some(value) and match the expected outcome.
            assert(dequeue_res =~= Some(value));

            let buf_len = buf.len();
            // After dequeue, the buffer should be empty.
            assert(buf_len == 0);

            let has_elements = buf.has_elements();
            // The buffer should report being empty.
            assert(!has_elements);
        }
    }
}