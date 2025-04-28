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
            let ring_seq = self.ring@;
            let capacity = ring_seq.len();
            let h = self.head;
            let t = self.tail;
            let occupant_count = if t >= h {
                (t - h) as nat
            } else {
                (capacity - h as nat) + t as nat
            };
            let occupant = Seq::from_fn(occupant_count, |i: int| {
                ring_seq.index(((h as int) + i) % capacity)
            });
            (occupant, capacity)
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
            let capacity = self.ring@.len();
            capacity > 1
            && self.head < capacity
            && self.tail < capacity
            && {
                let occupant_count = if self.tail >= self.head {
                    (self.tail - self.head) as nat
                } else {
                    (capacity - self.head + self.tail) as nat
                };
                occupant_count == self@.0.len() && occupant_count <= capacity - 1
            }
        }

        pub fn len(&self) -> (ret: usize)
            ensures
                ret == self@.0.len(),
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
            ensures
                ret == (self@.0.len() > 0),
        {
            proof {
                use_type_invariant(&self);
            }
            self.head != self.tail
        }

        pub fn is_full(&self) -> (ret: bool)
            ensures
                ret == (self@.0.len() == self@.1 - 1),
        {
            proof {
                use_type_invariant(&self);
                lemma_mod_auto(self@.1 as int);
            }
            self.head == ((self.tail + 1) % self.ring.len())
        }

        pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
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
            ensures
                if old(self)@.0.len() == old(self)@.1 - 1 {
                    self@.0.len() == old(self)@.0.len() && succ == false
                } else {
                    self@.0.len() == old(self)@.0.len() + 1 && succ == true
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
            ensures
                if old(self)@.0.len() > 0 {
                    ret.is_Some()
                    && ret.get_Some_0() == old(self)@.0.index(0)
                    && self@.0.len() == old(self)@.0.len() - 1
                } else {
                    ret == None
                    && self@.0.len() == old(self)@.0.len()
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
            ensures
                ret == self@.1 - self@.0.len() - 1,
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
// VerusErrorType.Other: `if` and `else` have incompatible typesVerusErrorType.Other: no function or associated item named `from_fn` found for struct `vstd::seq::Seq` in the current scopeVerusErrorType.MismatchedType: mismatched typesVerusErrorType.TypeAnnotation: type annotations needed
// {"$message_type":"diagnostic","message":"`if` and `else` have incompatible types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":941,"byte_end":973,"line_start":40,"line_end":40,"column_start":17,"column_end":49,"is_primary":true,"text":[{"text":"                (capacity - h as nat) + t as nat","highlight_start":17,"highlight_end":49}],"label":"expected `nat`, found `int`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":889,"byte_end":903,"line_start":38,"line_end":38,"column_start":17,"column_end":31,"is_primary":false,"text":[{"text":"                (t - h) as nat","highlight_start":17,"highlight_end":31}],"label":"expected because of this","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":861,"byte_end":987,"line_start":37,"line_end":41,"column_start":34,"column_end":14,"is_primary":false,"text":[{"text":"            let occupant_count = if t >= h {","highlight_start":34,"highlight_end":45},{"text":"                (t - h) as nat","highlight_start":1,"highlight_end":31},{"text":"            } else {","highlight_start":1,"highlight_end":21},{"text":"                (capacity - h as nat) + t as nat","highlight_start":1,"highlight_end":49},{"text":"            };","highlight_start":1,"highlight_end":14}],"label":"`if` and `else` have incompatible types","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0308]: `if` and `else` have incompatible types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug:40:17\n   |\n37 |               let occupant_count = if t >= h {\n   |  __________________________________-\n38 | |                 (t - h) as nat\n   | |                 -------------- expected because of this\n39 | |             } else {\n40 | |                 (capacity - h as nat) + t as nat\n   | |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `nat`, found `int`\n41 | |             };\n   | |_____________- `if` and `else` have incompatible types\n\n"}
// {"$message_type":"diagnostic","message":"no function or associated item named `from_fn` found for struct `vstd::seq::Seq` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":1021,"byte_end":1028,"line_start":42,"line_end":42,"column_start":33,"column_end":40,"is_primary":true,"text":[{"text":"            let occupant = Seq::from_fn(occupant_count, |i: int| {","highlight_start":33,"highlight_end":40}],"label":"function or associated item not found in `Seq<_>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"if you're trying to build a new `vstd::seq::Seq<_>` consider using one of the following associated functions:\nvstd::seq::Seq::<A>::empty\nvstd::seq::Seq::<A>::new","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":1379,"byte_end":1408,"line_start":38,"line_end":38,"column_start":5,"column_end":34,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/Users/runner/work/verus/verus/source/vstd/seq.rs","byte_start":1563,"byte_end":1620,"line_start":42,"line_end":42,"column_start":5,"column_end":62,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"there is an associated function `from` with a similar name","code":null,"level":"help","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/convert/mod.rs","byte_start":22708,"byte_end":22734,"line_start":585,"line_end":585,"column_start":5,"column_end":31,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no function or associated item named `from_fn` found for struct `vstd::seq::Seq` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug:42:33\n   |\n42 |             let occupant = Seq::from_fn(occupant_count, |i: int| {\n   |                                 ^^^^^^^ function or associated item not found in `Seq<_>`\n   |\nnote: if you're trying to build a new `vstd::seq::Seq<_>` consider using one of the following associated functions:\n      vstd::seq::Seq::<A>::empty\n      vstd::seq::Seq::<A>::new\n  --> /Users/runner/work/verus/verus/source/vstd/seq.rs:38:5\nhelp: there is an associated function `from` with a similar name\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/convert/mod.rs:585:5\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":1106,"byte_end":1114,"line_start":43,"line_end":43,"column_start":51,"column_end":59,"is_primary":true,"text":[{"text":"                ring_seq.index(((h as int) + i) % capacity)","highlight_start":51,"highlight_end":59}],"label":"expected `int`, found `nat`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":1087,"byte_end":1114,"line_start":43,"line_end":43,"column_start":32,"column_end":59,"is_primary":false,"text":[{"text":"                ring_seq.index(((h as int) + i) % capacity)","highlight_start":32,"highlight_end":59}],"label":"arguments to this method are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the return type of this call is `builtin::nat` due to the type of the argument passed","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":1106,"byte_end":1114,"line_start":43,"line_end":43,"column_start":51,"column_end":59,"is_primary":false,"text":[{"text":"                ring_seq.index(((h as int) + i) % capacity)","highlight_start":51,"highlight_end":59}],"label":"this argument influences the return type of `spec_euclidean_mod`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":1087,"byte_end":1114,"line_start":43,"line_end":43,"column_start":32,"column_end":59,"is_primary":true,"text":[{"text":"                ring_seq.index(((h as int) + i) % capacity)","highlight_start":32,"highlight_end":59}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":26717,"byte_end":26735,"line_start":1018,"line_end":1018,"column_start":8,"column_end":26,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug:43:51\n   |\n43 |                 ring_seq.index(((h as int) + i) % capacity)\n   |                                -------------------^^^^^^^^\n   |                                |                  |\n   |                                |                  expected `int`, found `nat`\n   |                                arguments to this method are incorrect\n   |\nhelp: the return type of this call is `builtin::nat` due to the type of the argument passed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug:43:32\n   |\n43 |                 ring_seq.index(((h as int) + i) % capacity)\n   |                                ^^^^^^^^^^^^^^^^^^^--------\n   |                                                   |\n   |                                                   this argument influences the return type of `spec_euclidean_mod`\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:1018:8\n\n"}
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":5630,"byte_end":5634,"line_start":200,"line_end":200,"column_start":28,"column_end":32,"is_primary":true,"text":[{"text":"                    ret == None","highlight_start":28,"highlight_end":32}],"label":"cannot infer type of the type parameter `T` declared on the enum `Option`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug","byte_start":5634,"byte_end":5634,"line_start":200,"line_end":200,"column_start":32,"column_end":32,"is_primary":true,"text":[{"text":"                    ret == None","highlight_start":32,"highlight_end":32}],"label":null,"suggested_replacement":"::<T>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj07__yug:200:28\n    |\n200 |                     ret == None\n    |                            ^^^^ cannot infer type of the type parameter `T` declared on the enum `Option`\n    |\nhelp: consider specifying the generic argument\n    |\n200 |                     ret == None::<T>\n    |                                +++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0282, E0308, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0282, E0308, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0282`.\n"}
//
//
