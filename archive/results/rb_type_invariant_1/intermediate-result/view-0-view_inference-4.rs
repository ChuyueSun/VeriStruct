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
            if self.tail >= self.head {
                (
                    self.ring@.subrange(self.head as int, self.tail as int),
                    capacity
                )
            } else {
                (
                    self.ring@.subrange(self.head as int, ( capacity ) as int)
                        + self.ring@.subrange(0, self.tail as int),
                    capacity
                )
            }
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
        closed spec fn inv(&self) -> bool {
            // TODO: specify the invariant
            true
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 17
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFailVecLen: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3025,"byte_end":3030,"line_start":99,"line_end":99,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3879,"byte_end":3909,"line_start":135,"line_end":135,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:135:17\n    |\n99  |             n > 0,\n    |             ----- failed precondition\n...\n135 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":4064,"byte_end":4093,"line_start":140,"line_end":140,"column_start":17,"column_end":46,"is_primary":true,"text":[{"text":"                (self.ring.len() - self.head) + self.tail","highlight_start":17,"highlight_end":46}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:140:17\n    |\n140 |                 (self.ring.len() - self.head) + self.tail\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":4064,"byte_end":4105,"line_start":140,"line_end":140,"column_start":17,"column_end":58,"is_primary":true,"text":[{"text":"                (self.ring.len() - self.head) + self.tail","highlight_start":17,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:140:17\n    |\n140 |                 (self.ring.len() - self.head) + self.tail\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3025,"byte_end":3030,"line_start":99,"line_end":99,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":4814,"byte_end":4844,"line_start":164,"line_end":164,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:164:17\n    |\n99  |             n > 0,\n    |             ----- failed precondition\n...\n164 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":4886,"byte_end":4901,"line_start":166,"line_end":166,"column_start":27,"column_end":42,"is_primary":true,"text":[{"text":"            self.head == ((self.tail + 1) % self.ring.len())","highlight_start":27,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:166:27\n    |\n166 |             self.head == ((self.tail + 1) % self.ring.len())\n    |                           ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3025,"byte_end":3030,"line_start":99,"line_end":99,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":5646,"byte_end":5676,"line_start":191,"line_end":191,"column_start":21,"column_end":51,"is_primary":true,"text":[{"text":"                    lemma_mod_auto(self@.1 as int);","highlight_start":21,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:191:21\n    |\n99  |             n > 0,\n    |             ----- failed precondition\n...\n191 |                     lemma_mod_auto(self@.1 as int);\n    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3224,"byte_end":3242,"line_start":110,"line_end":110,"column_start":13,"column_end":31,"is_primary":false,"text":[{"text":"            i < old(vec).len(),","highlight_start":13,"highlight_end":31}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":5712,"byte_end":5750,"line_start":193,"line_end":193,"column_start":17,"column_end":55,"is_primary":true,"text":[{"text":"                my_set(&mut self.ring, self.tail, val);","highlight_start":17,"highlight_end":55}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:193:17\n    |\n110 |             i < old(vec).len(),\n    |             ------------------ failed precondition\n...\n193 |                 my_set(&mut self.ring, self.tail, val);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":5780,"byte_end":5795,"line_start":194,"line_end":194,"column_start":29,"column_end":44,"is_primary":true,"text":[{"text":"                self.tail = (self.tail + 1) % self.ring.len();","highlight_start":29,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:194:29\n    |\n194 |                 self.tail = (self.tail + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":3025,"byte_end":3030,"line_start":99,"line_end":99,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":6110,"byte_end":6140,"line_start":205,"line_end":205,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:205:17\n    |\n99  |             n > 0,\n    |             ----- failed precondition\n...\n205 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/std_specs/vec.rs","byte_start":1740,"byte_end":1760,"line_start":52,"line_end":52,"column_start":9,"column_end":29,"is_primary":false,"text":[{"text":"        i < vec.view().len(),","highlight_start":9,"highlight_end":29}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":6220,"byte_end":6240,"line_start":209,"line_end":209,"column_start":27,"column_end":47,"is_primary":true,"text":[{"text":"                let val = self.ring[self.head];","highlight_start":27,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:209:27\n    |\n209 |                 let val = self.ring[self.head];\n    |                           ^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/std_specs/vec.rs:52:9\n    |\n52  |         i < vec.view().len(),\n    |         -------------------- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":6270,"byte_end":6285,"line_start":210,"line_end":210,"column_start":29,"column_end":44,"is_primary":true,"text":[{"text":"                self.head = (self.head + 1) % self.ring.len();","highlight_start":29,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:210:29\n    |\n210 |                 self.head = (self.head + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":6715,"byte_end":6729,"line_start":225,"line_end":225,"column_start":44,"column_end":58,"is_primary":true,"text":[{"text":"            self.ring.len().saturating_sub(1 + self.len())","highlight_start":44,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:225:44\n    |\n225 |             self.ring.len().saturating_sub(1 + self.len())\n    |                                            ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":6791,"byte_end":6865,"line_start":230,"line_end":230,"column_start":5,"column_end":79,"is_primary":true,"text":[{"text":"    fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":5,"highlight_end":79}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:230:5\n    |\n230 |     fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":7287,"byte_end":7297,"line_start":250,"line_end":250,"column_start":16,"column_end":26,"is_primary":true,"text":[{"text":"        assert(buf@.1 > 1);","highlight_start":16,"highlight_end":26}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:250:16\n    |\n250 |         assert(buf@.1 > 1);\n    |                ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":7374,"byte_end":7391,"line_start":254,"line_end":254,"column_start":17,"column_end":34,"is_primary":true,"text":[{"text":"                buf@.0.len() == 0,","highlight_start":17,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:254:17\n    |\n254 |                 buf@.0.len() == 0,\n    |                 ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":7499,"byte_end":7510,"line_start":258,"line_end":258,"column_start":20,"column_end":31,"is_primary":true,"text":[{"text":"            assert(enqueue_res);","highlight_start":20,"highlight_end":31}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:258:20\n    |\n258 |             assert(enqueue_res);\n    |                    ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":7570,"byte_end":7582,"line_start":261,"line_end":261,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(buf_len == 1);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:261:20\n    |\n261 |             assert(buf_len == 1);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2","byte_start":7656,"byte_end":7668,"line_start":264,"line_end":264,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(has_elements);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwjk9mnj2:264:20\n    |\n264 |             assert(has_elements);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 17 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 17 previous errors\n\n"}
//
//
