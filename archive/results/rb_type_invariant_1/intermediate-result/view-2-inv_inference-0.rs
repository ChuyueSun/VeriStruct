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
            let ring_len = self.ring@.len();
            let contents = if self.tail >= self.head {
                self.ring@.subrange(self.head, self.tail)
            } else {
                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)
            };
            (contents, ring_len)
        }
    }

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
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            self.ring@.len() > 1
            && self.head < self.ring@.len()
            && self.tail < self.ring@.len()
            && (if self.tail >= self.head {
                self.tail - self.head < self.ring@.len()
            } else {
                self.ring@.len() - self.head + self.tail < self.ring@.len()
            })
        }

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

        pub fn has_elements(&self) -> (ret: bool)
        {
            proof {
                use_type_invariant(&self);
            }
            self.head != self.tail
        }

        pub fn is_full(&self) -> (ret: bool)
        {
            proof {
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        {
            RingBuffer {
                head: 0,
                tail: 0,
                ring,
            }
        }

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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: arguments to this method are incorrectVerusErrorType.Other: arguments to this method are incorrectVerusErrorType.MismatchedType: mismatched types
// {"$message_type":"diagnostic","message":"arguments to this method are incorrect","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":820,"byte_end":829,"line_start":35,"line_end":35,"column_start":37,"column_end":46,"is_primary":false,"text":[{"text":"                self.ring@.subrange(self.head, self.tail)","highlight_start":37,"highlight_end":46}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":831,"byte_end":840,"line_start":35,"line_end":35,"column_start":48,"column_end":57,"is_primary":false,"text":[{"text":"                self.ring@.subrange(self.head, self.tail)","highlight_start":48,"highlight_end":57}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":811,"byte_end":819,"line_start":35,"line_end":35,"column_start":28,"column_end":36,"is_primary":true,"text":[{"text":"                self.ring@.subrange(self.head, self.tail)","highlight_start":28,"highlight_end":36}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: arguments to this method are incorrect\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6:35:28\n   |\n35 |                 self.ring@.subrange(self.head, self.tail)\n   |                            ^^^^^^^^ ---------  --------- expected `int`, found `usize`\n   |                                     |\n   |                                     expected `int`, found `usize`\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"arguments to this method are incorrect","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":899,"byte_end":908,"line_start":37,"line_end":37,"column_start":37,"column_end":46,"is_primary":false,"text":[{"text":"                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)","highlight_start":37,"highlight_end":46}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":910,"byte_end":918,"line_start":37,"line_end":37,"column_start":48,"column_end":56,"is_primary":false,"text":[{"text":"                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)","highlight_start":48,"highlight_end":56}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":890,"byte_end":898,"line_start":37,"line_end":37,"column_start":28,"column_end":36,"is_primary":true,"text":[{"text":"                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)","highlight_start":28,"highlight_end":36}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: arguments to this method are incorrect\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6:37:28\n   |\n37 |                 self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)\n   |                            ^^^^^^^^ ---------  -------- expected `int`, found `nat`\n   |                                     |\n   |                                     expected `int`, found `usize`\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":945,"byte_end":954,"line_start":37,"line_end":37,"column_start":83,"column_end":92,"is_primary":true,"text":[{"text":"                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)","highlight_start":83,"highlight_end":92}],"label":"expected `int`, found `usize`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6","byte_start":933,"byte_end":941,"line_start":37,"line_end":37,"column_start":71,"column_end":79,"is_primary":false,"text":[{"text":"                self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)","highlight_start":71,"highlight_end":79}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":3836,"byte_end":3844,"line_start":114,"line_end":114,"column_start":17,"column_end":25,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp2ieb7hf6:37:83\n   |\n37 |                 self.ring@.subrange(self.head, ring_len) + self.ring@.subrange(0, self.tail)\n   |                                                                       --------    ^^^^^^^^^ expected `int`, found `usize`\n   |                                                                       |\n   |                                                                       arguments to this method are incorrect\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:114:17\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0308`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0308`.\n"}
//
//
