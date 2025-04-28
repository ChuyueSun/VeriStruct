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
        type V = Seq<T>;

        closed spec fn view(&self) -> Self::V {
            if self.tail >= self.head {
                self.ring@.subrange(self.head as int, self.tail as int)
            } else {
                self.ring@.subrange(self.head as int, ( self.ring@.len() ) as int)
                    + self.ring@.subrange(0, self.tail as int)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 7
// VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<T>`VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<T>`VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<T>`VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<T>`VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<i32>`VerusErrorType.Other: no field `0` on type `vstd::seq::Seq<i32>`VerusErrorType.Other: no field `1` on type `vstd::seq::Seq<i32>`
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<T>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":3612,"byte_end":3613,"line_start":128,"line_end":128,"column_start":34,"column_end":35,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":34,"highlight_end":35}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<T>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:128:34\n    |\n128 |             lemma_mod_auto(self@.1 as int);\n    |                                  ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<T>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":4439,"byte_end":4440,"line_start":157,"line_end":157,"column_start":34,"column_end":35,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":34,"highlight_end":35}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<T>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:157:34\n    |\n157 |             lemma_mod_auto(self@.1 as int);\n    |                                  ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<T>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":5179,"byte_end":5180,"line_start":184,"line_end":184,"column_start":38,"column_end":39,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":38,"highlight_end":39}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<T>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:184:38\n    |\n184 |                 lemma_mod_auto(self@.1 as int);\n    |                                      ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<T>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":5591,"byte_end":5592,"line_start":198,"line_end":198,"column_start":34,"column_end":35,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":34,"highlight_end":35}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<T>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:198:34\n    |\n198 |             lemma_mod_auto(self@.1 as int);\n    |                                  ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<i32>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":6605,"byte_end":6606,"line_start":244,"line_end":244,"column_start":17,"column_end":18,"is_primary":true,"text":[{"text":"    assert(buf@.1 > 1);","highlight_start":17,"highlight_end":18}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<i32>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:244:17\n    |\n244 |     assert(buf@.1 > 1);\n    |                 ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `0` on type `vstd::seq::Seq<i32>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":6680,"byte_end":6681,"line_start":248,"line_end":248,"column_start":18,"column_end":19,"is_primary":true,"text":[{"text":"            buf@.0.len() == 0,","highlight_start":18,"highlight_end":19}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `0` on type `vstd::seq::Seq<i32>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:248:18\n    |\n248 |             buf@.0.len() == 0,\n    |                  ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"no field `1` on type `vstd::seq::Seq<i32>`","code":{"code":"E0609","explanation":"Attempted to access a nonexistent field in a struct.\n\nErroneous code example:\n\n```compile_fail,E0609\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.foo); // error: no field `foo` on type `StructWithFields`\n```\n\nTo fix this error, check that you didn't misspell the field's name or that the\nfield actually exists. Example:\n\n```\nstruct StructWithFields {\n    x: u32,\n}\n\nlet s = StructWithFields { x: 0 };\nprintln!(\"{}\", s.x); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9","byte_start":6711,"byte_end":6712,"line_start":249,"line_end":249,"column_start":18,"column_end":19,"is_primary":true,"text":[{"text":"            buf@.1 > 1","highlight_start":18,"highlight_end":19}],"label":"unknown field","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0609]: no field `1` on type `vstd::seq::Seq<i32>`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv50epee9:249:18\n    |\n249 |             buf@.1 > 1\n    |                  ^ unknown field\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 7 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 7 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0609`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0609`.\n"}
//
//
