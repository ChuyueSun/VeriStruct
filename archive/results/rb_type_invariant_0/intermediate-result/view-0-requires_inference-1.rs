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
            let len = self.ring@.len();
            let length = if self.tail >= self.head {
                (self.tail - self.head) as nat
            } else {
                (len - self.head + self.tail) as nat
            };
            (
                Seq::new(length, |i: int| {
                    self.ring@[((self.head as int + i) % ( len ) as int) as int]
                }),
                len as nat
            )
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
            &&& self.head < self.ring.len()
            &&& self.tail < self.ring.len()
            &&& self.ring.len() > 1
        }

        pub fn len(&self) -> (ret: usize)
            requires
                // no specific requirements
            ensures
                ret as int == self@.0.len(),
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
            requires
                // no specific requirements
            ensures
                ret <==> (self@.0.len() > 0),
        {
            proof {
                use_type_invariant(&self);
            }
            self.head != self.tail
        }

        pub fn is_full(&self) -> (ret: bool)
            requires
                // no specific requirements
            ensures
                ret <==> self@.0.len() == self@.1 - 1,
        {
            proof {
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
            requires
                // no specific requirements
            ensures
                ret@.0.len() == 0,
                ret@.1 == ring@.len(),
        {
            RingBuffer {
                head: 0,
                tail: 0,
                ring,
            }
        }

        pub fn enqueue(&mut self, val: T) -> (succ: bool)
            requires
                // no specific requirements
            ensures
                self@.1 == old(self)@.1,
                if old(self)@.0.len() == old(self)@.1 - 1 {
                    succ == false &&
                    self@ =~= old(self)@
                } else {
                    succ == true &&
                    self@.0 == old(self)@.0.push(val)
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

        pub fn dequeue(&mut self) -> (ret: Option<T>)
            requires
                // no specific requirements
            ensures
                self@.1 == old(self)@.1,
                if old(self)@.0.len() == 0 {
                    ret == None &&
                    self@ =~= old(self)@
                } else {
                    ret == Some(old(self)@.0.index(0)) &&
                    self@.0 == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
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

        pub fn available_len(&self) -> (ret: usize)
            requires
                // no specific requirements
            ensures
                if self@.0.len() + 1 < self@.1 {
                    ret as int == self@.1 - (self@.0.len() + 1)
                } else {
                    ret as int == 0
                },
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
// VerusErrorType.TypeAnnotation: type annotations needed
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp7dxt9t26","byte_start":5547,"byte_end":5551,"line_start":202,"line_end":202,"column_start":28,"column_end":32,"is_primary":true,"text":[{"text":"                    ret == None &&","highlight_start":28,"highlight_end":32}],"label":"cannot infer type of the type parameter `T` declared on the enum `Option`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp7dxt9t26","byte_start":5551,"byte_end":5551,"line_start":202,"line_end":202,"column_start":32,"column_end":32,"is_primary":true,"text":[{"text":"                    ret == None &&","highlight_start":32,"highlight_end":32}],"label":null,"suggested_replacement":"::<T>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp7dxt9t26:202:28\n    |\n202 |                     ret == None &&\n    |                            ^^^^ cannot infer type of the type parameter `T` declared on the enum `Option`\n    |\nhelp: consider specifying the generic argument\n    |\n202 |                     ret == None::<T> &&\n    |                                +++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0282`.\n"}
//
//
