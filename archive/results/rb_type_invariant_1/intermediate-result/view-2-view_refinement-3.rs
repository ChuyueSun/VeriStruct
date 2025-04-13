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
            let ring = self.ring@;
            let capacity = ring.len();
            if self.tail >= self.head {
                (ring.subrange(self.head, self.tail), capacity)
            } else {
                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: arguments to this method are incorrectVerusErrorType.Other: arguments to this method are incorrectVerusErrorType.MismatchedType: mismatched types
// {"$message_type":"diagnostic","message":"arguments to this method are incorrect","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":829,"byte_end":838,"line_start":36,"line_end":36,"column_start":32,"column_end":41,"is_primary":false,"text":[{"text":"                (ring.subrange(self.head, self.tail), capacity)","highlight_start":32,"highlight_end":41}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":840,"byte_end":849,"line_start":36,"line_end":36,"column_start":43,"column_end":52,"is_primary":false,"text":[{"text":"                (ring.subrange(self.head, self.tail), capacity)","highlight_start":43,"highlight_end":52}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":820,"byte_end":828,"line_start":36,"line_end":36,"column_start":23,"column_end":31,"is_primary":true,"text":[{"text":"                (ring.subrange(self.head, self.tail), capacity)","highlight_start":23,"highlight_end":31}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: arguments to this method are incorrect\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu:36:23\n   |\n36 |                 (ring.subrange(self.head, self.tail), capacity)\n   |                       ^^^^^^^^ ---------  --------- expected `int`, found `usize`\n   |                                |\n   |                                expected `int`, found `usize`\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"arguments to this method are incorrect","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":914,"byte_end":923,"line_start":38,"line_end":38,"column_start":32,"column_end":41,"is_primary":false,"text":[{"text":"                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)","highlight_start":32,"highlight_end":41}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":925,"byte_end":933,"line_start":38,"line_end":38,"column_start":43,"column_end":51,"is_primary":false,"text":[{"text":"                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)","highlight_start":43,"highlight_end":51}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":905,"byte_end":913,"line_start":38,"line_end":38,"column_start":23,"column_end":31,"is_primary":true,"text":[{"text":"                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)","highlight_start":23,"highlight_end":31}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: arguments to this method are incorrect\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu:38:23\n   |\n38 |                 (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)\n   |                       ^^^^^^^^ ---------  -------- expected `int`, found `nat`\n   |                                |\n   |                                expected `int`, found `usize`\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":954,"byte_end":963,"line_start":38,"line_end":38,"column_start":72,"column_end":81,"is_primary":true,"text":[{"text":"                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)","highlight_start":72,"highlight_end":81}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu","byte_start":942,"byte_end":950,"line_start":38,"line_end":38,"column_start":60,"column_end":68,"is_primary":false,"text":[{"text":"                (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)","highlight_start":60,"highlight_end":68}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpi52v96bu:38:72\n   |\n38 |                 (ring.subrange(self.head, capacity) + ring.subrange(0, self.tail), capacity)\n   |                                                            --------    ^^^^^^^^^ expected `int`, found `usize`\n   |                                                            |\n   |                                                            arguments to this method are incorrect\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0308`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0308`.\n"}
// 
// 