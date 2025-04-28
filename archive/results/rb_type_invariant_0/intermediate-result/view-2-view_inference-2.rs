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
            let cap = self.ring@.len();
            let content = if self.tail >= self.head {
                self.ring@[( self.head..self.tail ) as int]
            } else {
                self.ring@[self.head..cap] + self.ring@[0..self.tail]
            };
            (content, cap)
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
        {
            proof {
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        /// Creates a new RingBuffer with the given backing `ring` storage.
        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5
// VerusErrorType.Other: the trait bound `std::ops::Range<usize>: std::marker::Copy` is not satisfiedVerusErrorType.MismatchedType: mismatched typesVerusErrorType.MismatchedType: mismatched typesVerusErrorType.Other: no method named `spec_add` found for type parameter `T` in the current scopeVerusErrorType.MismatchedType: mismatched types
// {"$message_type":"diagnostic","message":"the trait bound `std::ops::Range<usize>: std::marker::Copy` is not satisfied","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":805,"byte_end":836,"line_start":35,"line_end":35,"column_start":28,"column_end":59,"is_primary":true,"text":[{"text":"                self.ring@[( self.head..self.tail ) as int]","highlight_start":28,"highlight_end":59}],"label":"the trait `std::marker::Copy` is not implemented for `std::ops::Range<usize>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required by a bound in `builtin::spec_cast_integer`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":24453,"byte_end":24527,"line_start":934,"line_end":934,"column_start":1,"column_end":75,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: the trait bound `std::ops::Range<usize>: std::marker::Copy` is not satisfied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm:35:28\n   |\n35 |                 self.ring@[( self.head..self.tail ) as int]\n   |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `std::marker::Copy` is not implemented for `std::ops::Range<usize>`\n   |\nnote: required by a bound in `builtin::spec_cast_integer`\n  --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:934:1\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":897,"byte_end":900,"line_start":37,"line_end":37,"column_start":39,"column_end":42,"is_primary":true,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":39,"highlight_end":42}],"label":"expected `usize`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm:37:39\n   |\n37 |                 self.ring@[self.head..cap] + self.ring@[0..self.tail]\n   |                                       ^^^ expected `usize`, found `nat`\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":886,"byte_end":900,"line_start":37,"line_end":37,"column_start":28,"column_end":42,"is_primary":true,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":28,"highlight_end":42}],"label":"expected `int`, found `Range<usize>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":875,"byte_end":901,"line_start":37,"line_end":37,"column_start":17,"column_end":43,"is_primary":false,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":17,"highlight_end":43}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"expected struct `builtin::int`\n   found struct `std::ops::Range<usize>`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":2178,"byte_end":2188,"line_start":60,"line_end":60,"column_start":22,"column_end":32,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm:37:28\n   |\n37 |                 self.ring@[self.head..cap] + self.ring@[0..self.tail]\n   |                 -----------^^^^^^^^^^^^^^-\n   |                 |          |\n   |                 |          expected `int`, found `Range<usize>`\n   |                 arguments to this method are incorrect\n   |\n   = note: expected struct `builtin::int`\n              found struct `std::ops::Range<usize>`\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:60:22\n\n"}
// {"$message_type":"diagnostic","message":"no method named `spec_add` found for type parameter `T` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":875,"byte_end":928,"line_start":37,"line_end":37,"column_start":17,"column_end":70,"is_primary":true,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":17,"highlight_end":70}],"label":"method not found in `T`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":569,"byte_end":570,"line_start":29,"line_end":29,"column_start":10,"column_end":11,"is_primary":false,"text":[{"text":"    impl<T: Copy> View for RingBuffer<T> {","highlight_start":10,"highlight_end":11}],"label":"method `spec_add` not found for this type parameter","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":884,"byte_end":885,"line_start":37,"line_end":37,"column_start":26,"column_end":27,"is_primary":false,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":26,"highlight_end":27}],"label":"method `spec_add` is available on `vstd::seq::Seq<T>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the type parameter is bounded by the trait","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following trait defines an item `spec_add`, perhaps you need to restrict type parameter `T` with it:","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":576,"byte_end":576,"line_start":29,"line_end":29,"column_start":17,"column_end":17,"is_primary":true,"text":[{"text":"    impl<T: Copy> View for RingBuffer<T> {","highlight_start":17,"highlight_end":17}],"label":null,"suggested_replacement":" + builtin::SpecAdd","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `spec_add` found for type parameter `T` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm:37:17\n   |\n29 |     impl<T: Copy> View for RingBuffer<T> {\n   |          - method `spec_add` not found for this type parameter\n...\n37 |                 self.ring@[self.head..cap] + self.ring@[0..self.tail]\n   |                 ^^^^^^^^^-^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n   |                 |        |\n   |                 |        method `spec_add` is available on `vstd::seq::Seq<T>`\n   |                 method not found in `T`\n   |\n   = help: items from traits can only be used if the type parameter is bounded by the trait\nhelp: the following trait defines an item `spec_add`, perhaps you need to restrict type parameter `T` with it:\n   |\n29 |     impl<T: Copy + builtin::SpecAdd> View for RingBuffer<T> {\n   |                  ++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":915,"byte_end":927,"line_start":37,"line_end":37,"column_start":57,"column_end":69,"is_primary":true,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":57,"highlight_end":69}],"label":"expected `int`, found `Range<usize>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm","byte_start":904,"byte_end":928,"line_start":37,"line_end":37,"column_start":46,"column_end":70,"is_primary":false,"text":[{"text":"                self.ring@[self.head..cap] + self.ring@[0..self.tail]","highlight_start":46,"highlight_end":70}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"expected struct `builtin::int`\n   found struct `std::ops::Range<usize>`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":2178,"byte_end":2188,"line_start":60,"line_end":60,"column_start":22,"column_end":32,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpo9xjkbpm:37:57\n   |\n37 |                 self.ring@[self.head..cap] + self.ring@[0..self.tail]\n   |                                              -----------^^^^^^^^^^^^-\n   |                                              |          |\n   |                                              |          expected `int`, found `Range<usize>`\n   |                                              arguments to this method are incorrect\n   |\n   = note: expected struct `builtin::int`\n              found struct `std::ops::Range<usize>`\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:60:22\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 5 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 5 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0277, E0308, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0277, E0308, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0277`.\n"}
//
//
