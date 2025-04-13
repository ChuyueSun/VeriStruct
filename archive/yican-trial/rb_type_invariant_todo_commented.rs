// Import the vstd prelude which brings in various verification utilities.
use vstd::prelude::*;

// The main function is empty; it's required for executable programs.
pub fn main() {}

verus! {
    ////////////////////////////////////////////////////////////////////////////////
    // Specification function for saturating subtraction.
    // This function is a pure mathematical specification (spec) that models
    // the behavior of saturating subtraction. It returns a natural number (nat)
    // describing the result of a subtract operation where if a > b then a - b,
    // otherwise 0.
    //
    // Precondition: (none; it's a spec function that just describes the behavior)
    // Postcondition: Returns (a - b) as a nat if a > b, or 0 otherwise.
    pub open spec fn ex_saturating_sub_spec(a: int, b: int) -> (ret: nat)
    {
        // If a is greater than b, subtract b from a and cast the result to a nat.
        if (a > b) {
            (a - b) as nat
        } else {
            // Otherwise, if a is not greater than b, the result is 0.
            0
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Executable function with external specification of saturating subtraction.
    #[verifier::external_fn_specification]
    pub fn ex_saturating_sub(a: usize, b: usize) -> (ret: usize)
    // Postcondition:
    //   Ensures that the execution of ex_saturating_sub returns the same result 
    //   as specified by ex_saturating_sub_spec with the proper cast conversions.
    ensures
        ex_saturating_sub_spec(a as int, b as int) == ret as int
    {
        // Call Rust's built-in saturating_sub for usize, which performs saturating subtraction.
        a.saturating_sub(b)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Definition of a RingBuffer data structure parameterized by a type T that is Copy.
    pub struct RingBuffer<T: Copy> {
        // The underlying storage vector that represents the ring.
        ring: Vec<T>,
        // The current head position (index) of the ring buffer.
        head: usize,
        // The current tail position (index) of the ring buffer.
        tail: usize,
    }

    ////////////////////////////////////////////////////////////////////////////////
    // The View trait provides a way to expose a logical view of the RingBuffer.
    impl<T: Copy> View for RingBuffer<T> {
        // The logical view is represented as a tuple:
        //   - First element: a sequence (Seq<T>) of the current elements in the buffer.
        //   - Second element: the capacity of the buffer (the length of the underlying ring vector).
        type V = (Seq<T>, usize);
    
        // Specification function that returns the logical view of the RingBuffer.
        // It extracts the elements currently in use from the ring vector based on head and tail,
        // and provides the current capacity.
        closed spec fn view(&self) -> Self::V {
            // Obtain the capacity of the ring buffer.
            let cap = self.ring.len();
            // If the tail index is greater than or equal to the head index, the active region is contiguous.
            if self.tail >= self.head {
                (
                    // The active elements are the subrange from head to tail.
                    (self.ring)@.subrange(self.head as int, self.tail as int),
                    cap
                )
            } else {
                (
                    // Otherwise, the active region wraps around the end of the ring.
                    // Concatenate the subrange from head to the end with the subrange from start to tail.
                    (self.ring)@.subrange(self.head as int, cap as int)
                        .add((self.ring)@.subrange(0, self.tail as int)),
                    cap
                )
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Specification function that states a useful property for the modulo operation.
    // For any integers x and y, when using modulo with a positive divisor n, the sum:
    //   (x % n) + (y % n)
    // must fall into one of two cases:
    //   (1) If the sum z is in the range [0, n), then z equals (x + y) % n.
    //   (2) If the sum z is in the range [n, 2n), then z equals (x + y) % n + n.
    // The function returns a boolean representing that this property holds.
    //
    // Recommended condition: n > 0.
    pub open spec fn mod_auto_plus(n: int) -> bool
        recommends
            n > 0, // Recommends that n is strictly greater than zero.
    {
        // For all integers x and y...
        forall|x: int, y: int|
            {
                // Compute the sum of the mods.
                let z = (x % n) + (y % n);
                // The property holds if either:
                //   - z is in [0, n) and equals (x+y)%n, or
                //   - z is in [n, 2n) and equals (x+y)%n plus n.
                ((0 <= z < n && #[trigger] ((x + y) % n) == z)
                    || (n <= z < n + n && ((x + y) % n) == z - n))
            }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Specification function that states a useful property for the modulo operation with subtraction.
    // For any integers x and y, when using modulo with a positive divisor n, the difference:
    //   (x % n) - (y % n)
    // must fall into one of two cases:
    //   (1) If the difference z is in the range [0, n), then z equals (x - y) % n.
    //   (2) If the difference z is in the range [-n, 0), then z equals (x - y) % n - n.
    // The function returns a boolean representing that this property holds.
    //
    // Recommended condition: n > 0.
    pub open spec fn mod_auto_minus(n: int) -> bool
        recommends
            n > 0, // Recommends that n is strictly greater than zero.
    {
        // For all integers x and y...
        forall|x: int, y: int|
            {
                // Compute the difference of the mods.
                let z = (x % n) - (y % n);
                // The property holds if either:
                //   - z is in [0, n) and equals (x-y)%n, or
                //   - z is in [-n, 0) and equals (x-y)%n plus n.
                ((0 <= z < n && #[trigger] ((x - y) % n) == z)
                    || (-n <= z < 0 && ((x - y) % n) == z + n))
            }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Specification function that aggregates several useful properties of the modulo operation.
    // When the divisor is a positive number n, the following properties are stated:
    //   - n % n equals 0.
    //   - (-n) % n equals 0.
    //   - For any integer x, (x % n) % n equals x % n.
    //   - For any integer x, if 0 <= x < n then x % n equals x.
    //   - The properties of mod_auto_plus(n) hold.
    //   - The properties of mod_auto_minus(n) hold.
    //
    // Recommended condition: n > 0.
    pub open spec fn mod_auto(n: int) -> bool
        recommends
            n > 0, // Recommends that n is strictly greater than zero.
    {
        // Conjunction (logical AND) of several modulo properties.
        &&& (n % n == 0 && (-n) % n == 0)
        &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
        &&& (forall|x: int| 0 <= x < n <==> #[trigger] (x % n) == x)
        &&& mod_auto_plus(n)
        &&& mod_auto_minus(n)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Proof function for mod_auto.
    // This proof function states that when n > 0, the specification mod_auto(n)
    // holds. Essentially, it assures the verifier that all modulo properties
    // defined in mod_auto hold for any positive n.
    //
    // Precondition:
    //   requires n > 0, meaning n must be strictly positive.
    // Postcondition:
    //   ensures mod_auto(n) holds.
    pub proof fn lemma_mod_auto(n: int)
        requires
            n > 0, // The divisor n must be positive.
        ensures
            mod_auto(n), // Proves that all properties in mod_auto(n) hold.
    {
        // Here, admit() is used as a placeholder to indicate that the proof is left
        // unimplemented. In a full verification, this would be replaced by actual proof steps.
        admit()
    }

    ////////////////////////////////////////////////////////////////////////////////
    // External body function that sets an element in a vector at index i with value.
    #[verifier::external_body]
    fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
        requires
            // Precondition: The index i is within bounds of the original vector.
            i < old(vec).len(),
        ensures
            // Postcondition: The logical view (vec@) of the vector after modification
            // equals the old view with the element at index i updated to value.
            vec@ == old(vec)@.update(i as int, value),
            // The length of the vector remains unchanged.
            vec@.len() == old(vec).len()
            no_unwind
    {
        // Set the element at the specified index in the vector.
        vec[i] = value;
    }


    impl<T: Copy> RingBuffer<T> {
        ////////////////////////////////////////////////////////////////////////////////
        // Invariant for the ring buffer data structure.
        // This type invariant must hold at all times for a valid RingBuffer object.
        // It asserts that:
        //   - head is within the bounds of the underlying ring vector.
        //   - tail is within the bounds of the underlying ring vector.
        //   - The ring vector has at least one element.
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            &&& self.head < self.ring.len()
            &&& self.tail < self.ring.len()
            &&& self.ring.len() > 0
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Returns the number of elements currently stored in the ring buffer.
        pub fn len(&self) -> (ret: usize)
        // TODO: add this
        {
            // Begin a verification proof block to assert the ring buffer invariant holds.
            proof {
                // Use the type invariant of the ring buffer.
                use_type_invariant(&self);
            }
            // If tail > head, the elements are contiguous.
            if self.tail > self.head {
                // The number of elements is just the difference.
                self.tail - self.head
            } else if self.tail < self.head {
                // If the elements wrap around: count from head to the end plus from start to tail.
                (self.ring.len() - self.head) + self.tail
            } else {
                // If head equals tail, the buffer is empty.
                0
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Returns true if there is at least one element in the buffer.
        pub fn has_elements(&self) -> (ret: bool)
        // TODO: add this
        {
            proof {
                // Assert that the ring buffer invariant holds.
                use_type_invariant(&self);
            }
            // The buffer has elements if head and tail are not equal.
            self.head != self.tail
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Returns true if the buffer is full.
        // The buffer is considered full when the logical length equals (capacity - 1).
        pub fn is_full(&self) -> (ret: bool)
        // TODO: add this
        {
            proof {
                // Assert that the ring buffer invariant holds.
                use_type_invariant(&self);
                // Prove properties about modulo arithmetic using lemma_mod_auto.
                lemma_mod_auto(self@.1 as int);
            }
            // Check fullness condition: if head equals (tail+1) modulo the capacity,
            // then the buffer is full.
            self.head == ((self.tail + 1) % self.ring.len())
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Constructs a new RingBuffer from the given backing vector.
        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        // TODO: add this
        {
            // Create a new RingBuffer object with head and tail set to 0.
            RingBuffer {
                head: 0,
                tail: 0,
                ring,
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Enqueues an element at the back of the ring buffer if it is not full.
        pub fn enqueue(&mut self, val: T) -> (succ: bool)
        // TODO: add this
        {
            // Check if the buffer is full.
            if self.is_full() {
                // If it is full, then the element cannot be enqueued. Return false.
                false
            } else {
                proof {
                    // Assert that the type invariant holds.
                    use_type_invariant(&*self);
                    // Use the lemma for modulo arithmetic to help with verifying wrap-around logic.
                    lemma_mod_auto(self@.1 as int);
                }
                // Update the ring buffer's storage at index tail with the new value.
                my_set(&mut self.ring, self.tail, val);
                // Advance the tail index, wrapping around using modulo operation.
                self.tail = (self.tail + 1) % self.ring.len();
                // Return true indicating that the value was enqueued successfully.
                true
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Dequeues an element from the front of the ring buffer.
        pub fn dequeue(&mut self) -> (ret: Option<T>)
        // TODO: add this
        {
            proof {
                // Assert that the type invariant is maintained.
                use_type_invariant(&*self);
                // Use modulo arithmetic lemma to help verify operation correctness.
                lemma_mod_auto(self@.1 as int);
            }

            // Check if the buffer has any elements.
            if self.has_elements() {
                // Retrieve the element at the head of the buffer.
                let val = self.ring[self.head];
                // Advance the head pointer using modulo to wrap around if necessary.
                self.head = (self.head + 1) % self.ring.len();
                // Return the retrieved element wrapped in Some.
                Some(val)
            } else {
                // If the buffer is empty, return None.
                None
            }
        }
        
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Test function to verify the behavior of enqueue and dequeue operations on a RingBuffer.
    // The function is generic over different buffer lengths, values, and iteration counts.
    // Precondition:
    //   - len is less than usize::MAX - 1 to avoid overflow in capacity calculations.
    //   - 2 * iterations fits within usize (to prevent iteration count overflow).
    #[verifier::loop_isolation(false)]
    fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)
        requires
            len < usize::MAX - 1, // Ensure that the ring capacity does not overflow.
            iterations * 2 < usize::MAX, // Ensure that the total iteration count is safe.
    {
        // Initialize an empty vector to be used as the backing storage for the ring buffer.
        let mut ring: Vec<i32> = Vec::new();

        // If the specified length is 0, then exit early as further operations do not make sense.
        if len == 0 {
            return;
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Increase the size of the vector to len + 1.
        // Loop invariant:
        //   At every iteration, the length of 'ring' equals the current loop variable i.
        for i in 0..(len + 1)
            invariant
                ring.len() == i, // Invariant: The length of 'ring' equals the iteration counter i.
        {
            // Push a default value (0) onto the vector to grow its size.
            ring.push(0);
        }

        // Assert that the ring vector now has more than one element.
        assert(ring.len() > 1);
        // Create a new ring buffer using the built-up vector.
        let mut buf = RingBuffer::new(ring);
        // Assert that the capacity (second component of the logical view) is greater than 1.
        assert(buf@.1 > 1);

        ////////////////////////////////////////////////////////////////////////////////
        // Run a loop for 2 * iterations to test the enqueue and dequeue operations repeatedly.
        // Loop invariants:
        //   - The logical view's sequence of elements (buf@.0) is empty,
        //     indicating the buffer is empty at the start of every iteration.
        //   - The capacity (buf@.1) is greater than 1.
        for _ in 0..2 * iterations
            invariant
                buf@.0.len() == 0, // Invariant: The buffer is empty at the beginning and end of each iteration.
                buf@.1 > 1        // Invariant: The capacity remains greater than 1.
        {
            // Attempt to enqueue the provided value; store the result.
            let enqueue_res = buf.enqueue(value);
            // Assert that enqueuing was successful.
            assert(enqueue_res);

            // Get the current length of the buffer.
            let buf_len = buf.len();
            // Assert that after enqueuing, the buffer length is exactly 1.
            assert(buf_len == 1);

            // Check whether the buffer has any elements.
            let has_elements = buf.has_elements();
            // Assert that the buffer correctly reports having elements.
            assert(has_elements);

            // Attempt to dequeue an element from the buffer.
            let dequeue_res = buf.dequeue();
            // Assert that the dequeued result matches the expected value.
            // The operator '=~=' is used to permit minor differences in representation.
            assert(dequeue_res =~= Some(value));

            // Again, get the buffer length after dequeuing.
            let buf_len = buf.len();
            // Assert that the buffer is now empty.
            assert(buf_len == 0);

            // Check again that the buffer reports no elements.
            let has_elements = buf.has_elements();
            // The buffer should now correctly report as not having any elements.
            assert(!has_elements);
        }
    }
}