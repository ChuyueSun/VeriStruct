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

        /// The view returns the current contents of the ring buffer (in order) and the capacity.
        closed spec fn view(&self) -> Self::V {
            (
                Seq::new(
                    (if self.tail >= self.head {
                        self.tail - self.head
                    } else {
                        self.ring@.len() - self.head + self.tail
                    }) as nat,
                    |i: int| {
                        self.ring@[
                            ((self.head as int + i) % (self.ring@.len() as int)) as int
                        ]
                    }
                ),
                self.ring@.len()
            )
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
            vec@.len() == old(vec).len(),
        ( no_unwind ) as bool
    {
        vec[i] = value;
    }

    impl<T: Copy> RingBuffer<T> {
        /// Invariant for the ring buffer.
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            &&& self.head < self.ring.len()
            &&& self.tail < self.ring.len()
        }

        /// Returns how many elements are in the buffer.
        pub fn len(&self) -> (ret: usize)
            requires
                self.inv(),
            ensures
                ret == self@.0.len(),
                self.inv(),
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
            requires
                self.inv(),
            ensures
                ret == (self@.0.len() > 0),
                self.inv(),
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
            requires
                self.inv(),
            ensures
                ret == (self@.0.len() == self.ring@.len() - 1),
                self.inv(),
        {
            proof {
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        /// Creates a new RingBuffer with the given backing `ring` storage.
        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
            requires
                ring@.len() > 0,
            ensures
                ret.inv(),
                ret@.0.len() == 0,
                ret@.1 == ring@.len(),
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
            requires
                self.inv(),
            ensures
                self.inv(),
                if !succ {
                    self@ == old(self)@
                } else {
                    self@.0.len() == old(self)@.0.len() + 1
                    && self@.1 == old(self)@.1
                    && self@.0.subrange(0, old(self)@.0.len()) =~= old(self)@.0
                    && self@.0.index(old(self)@.0.len()) == val
                },
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
            requires
                self.inv(),
            ensures
                self.inv(),
                match ret {
                    None => self@ == old(self)@,
                    Some(v) => {
                        self@.0.len() == old(self)@.0.len() - 1
                        && self@.1 == old(self)@.1
                        && old(self)@.0.index(0) == v
                        && self@.0 =~= old(self)@.0.subrange(1, old(self)@.0.len())
                    }
                },
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
            requires
                self.inv(),
            ensures
                ret == self@.1 - 1 - self@.0.len(),
                self.inv(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: the trait bound `bool: builtin::Integer` is not satisfiedVerusErrorType.MismatchedType: mismatched typesVerusErrorType.MismatchedType: mismatched typesVerusErrorType.MismatchedType: mismatched types
// {"$message_type":"diagnostic","message":"the trait bound `bool: builtin::Integer` is not satisfied","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":3534,"byte_end":3538,"line_start":116,"line_end":116,"column_start":26,"column_end":30,"is_primary":true,"text":[{"text":"        ( no_unwind ) as bool","highlight_start":26,"highlight_end":30}],"label":"the trait `builtin::Integer` is not implemented for `bool`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required by a bound in `builtin::spec_cast_integer`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":24453,"byte_end":24527,"line_start":934,"line_end":934,"column_start":1,"column_end":75,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: the trait bound `bool: builtin::Integer` is not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69:116:26\n    |\n116 |         ( no_unwind ) as bool\n    |                          ^^^^ the trait `builtin::Integer` is not implemented for `bool`\n    |\nnote: required by a bound in `builtin::spec_cast_integer`\n   --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:934:1\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":6376,"byte_end":6394,"line_start":209,"line_end":209,"column_start":44,"column_end":62,"is_primary":true,"text":[{"text":"                    && self@.0.subrange(0, old(self)@.0.len()) =~= old(self)@.0","highlight_start":44,"highlight_end":62}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":6364,"byte_end":6372,"line_start":209,"line_end":209,"column_start":32,"column_end":40,"is_primary":false,"text":[{"text":"                    && self@.0.subrange(0, old(self)@.0.len()) =~= old(self)@.0","highlight_start":32,"highlight_end":40}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69:209:44\n    |\n209 |                     && self@.0.subrange(0, old(self)@.0.len()) =~= old(self)@.0\n    |                                --------    ^^^^^^^^^^^^^^^^^^ expected `int`, found `nat`\n    |                                |\n    |                                arguments to this method are incorrect\n    |\nnote: method defined here\n   --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":6450,"byte_end":6468,"line_start":210,"line_end":210,"column_start":38,"column_end":56,"is_primary":true,"text":[{"text":"                    && self@.0.index(old(self)@.0.len()) == val","highlight_start":38,"highlight_end":56}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":6444,"byte_end":6449,"line_start":210,"line_end":210,"column_start":32,"column_end":37,"is_primary":false,"text":[{"text":"                    && self@.0.index(old(self)@.0.len()) == val","highlight_start":32,"highlight_end":37}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":2002,"byte_end":2007,"line_start":53,"line_end":53,"column_start":17,"column_end":22,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69:210:38\n    |\n210 |                     && self@.0.index(old(self)@.0.len()) == val\n    |                                ----- ^^^^^^^^^^^^^^^^^^ expected `int`, found `nat`\n    |                                |\n    |                                arguments to this method are incorrect\n    |\nnote: method defined here\n   --> /Users/runner/work/verus/verus/source/vstd/seq.rs:53:17\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":7441,"byte_end":7459,"line_start":238,"line_end":238,"column_start":65,"column_end":83,"is_primary":true,"text":[{"text":"                        && self@.0 =~= old(self)@.0.subrange(1, old(self)@.0.len())","highlight_start":65,"highlight_end":83}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69","byte_start":7429,"byte_end":7437,"line_start":238,"line_end":238,"column_start":53,"column_end":61,"is_primary":false,"text":[{"text":"                        && self@.0 =~= old(self)@.0.subrange(1, old(self)@.0.len())","highlight_start":53,"highlight_end":61}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpvm0zon69:238:65\n    |\n238 |                         && self@.0 =~= old(self)@.0.subrange(1, old(self)@.0.len())\n    |                                                     --------    ^^^^^^^^^^^^^^^^^^ expected `int`, found `nat`\n    |                                                     |\n    |                                                     arguments to this method are incorrect\n    |\nnote: method defined here\n   --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0277, E0308.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0277, E0308.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0277`.\n"}
// 
// 