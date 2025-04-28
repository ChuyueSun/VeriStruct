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
        type V = (Seq<T>, nat, nat);

        closed spec fn view(&self) -> Self::V {
            /* TODO: part of view */
            (self.ring@, self.head as nat, self.tail as nat)
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
        #[verifier::type_invariant]
        closed spec fn inv(&self) -> bool {
            // TODO: specify the invariant
            true
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 20
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.Other: possible division by zeroVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.Other: possible division by zeroVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFailVecLen: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.Other: possible division by zeroVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":3355,"byte_end":3385,"line_start":118,"line_end":118,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:118:17\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n118 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":3540,"byte_end":3569,"line_start":123,"line_end":123,"column_start":17,"column_end":46,"is_primary":true,"text":[{"text":"                (self.ring.len() - self.head) + self.tail","highlight_start":17,"highlight_end":46}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:123:17\n    |\n123 |                 (self.ring.len() - self.head) + self.tail\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":3540,"byte_end":3581,"line_start":123,"line_end":123,"column_start":17,"column_end":58,"is_primary":true,"text":[{"text":"                (self.ring.len() - self.head) + self.tail","highlight_start":17,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:123:17\n    |\n123 |                 (self.ring.len() - self.head) + self.tail\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":3963,"byte_end":3993,"line_start":141,"line_end":141,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:141:17\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n141 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4034,"byte_end":4069,"line_start":143,"line_end":143,"column_start":26,"column_end":61,"is_primary":true,"text":[{"text":"            self.head == ((self.tail + 1) % self.ring.len())","highlight_start":26,"highlight_end":61}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:143:26\n    |\n143 |             self.head == ((self.tail + 1) % self.ring.len())\n    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4035,"byte_end":4050,"line_start":143,"line_end":143,"column_start":27,"column_end":42,"is_primary":true,"text":[{"text":"            self.head == ((self.tail + 1) % self.ring.len())","highlight_start":27,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:143:27\n    |\n143 |             self.head == ((self.tail + 1) % self.ring.len())\n    |                           ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4505,"byte_end":4535,"line_start":162,"line_end":162,"column_start":21,"column_end":51,"is_primary":true,"text":[{"text":"                    lemma_mod_auto(self@.1 as int);","highlight_start":21,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:162:21\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n162 |                     lemma_mod_auto(self@.1 as int);\n    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":2852,"byte_end":2870,"line_start":98,"line_end":98,"column_start":13,"column_end":31,"is_primary":false,"text":[{"text":"            i < old(vec).len(),","highlight_start":13,"highlight_end":31}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4571,"byte_end":4609,"line_start":164,"line_end":164,"column_start":17,"column_end":55,"is_primary":true,"text":[{"text":"                my_set(&mut self.ring, self.tail, val);","highlight_start":17,"highlight_end":55}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:164:17\n    |\n98  |             i < old(vec).len(),\n    |             ------------------ failed precondition\n...\n164 |                 my_set(&mut self.ring, self.tail, val);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4639,"byte_end":4654,"line_start":165,"line_end":165,"column_start":29,"column_end":44,"is_primary":true,"text":[{"text":"                self.tail = (self.tail + 1) % self.ring.len();","highlight_start":29,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:165:29\n    |\n165 |                 self.tail = (self.tail + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4639,"byte_end":4672,"line_start":165,"line_end":165,"column_start":29,"column_end":62,"is_primary":true,"text":[{"text":"                self.tail = (self.tail + 1) % self.ring.len();","highlight_start":29,"highlight_end":62}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:165:29\n    |\n165 |                 self.tail = (self.tail + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4864,"byte_end":4894,"line_start":174,"line_end":174,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:174:17\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n174 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/std_specs/vec.rs","byte_start":1740,"byte_end":1760,"line_start":52,"line_end":52,"column_start":9,"column_end":29,"is_primary":false,"text":[{"text":"        i < vec.view().len(),","highlight_start":9,"highlight_end":29}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":4974,"byte_end":4994,"line_start":178,"line_end":178,"column_start":27,"column_end":47,"is_primary":true,"text":[{"text":"                let val = self.ring[self.head];","highlight_start":27,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:178:27\n    |\n178 |                 let val = self.ring[self.head];\n    |                           ^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/std_specs/vec.rs:52:9\n    |\n52  |         i < vec.view().len(),\n    |         -------------------- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5024,"byte_end":5039,"line_start":179,"line_end":179,"column_start":29,"column_end":44,"is_primary":true,"text":[{"text":"                self.head = (self.head + 1) % self.ring.len();","highlight_start":29,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:179:29\n    |\n179 |                 self.head = (self.head + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5024,"byte_end":5057,"line_start":179,"line_end":179,"column_start":29,"column_end":62,"is_primary":true,"text":[{"text":"                self.head = (self.head + 1) % self.ring.len();","highlight_start":29,"highlight_end":62}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:179:29\n    |\n179 |                 self.head = (self.head + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5334,"byte_end":5348,"line_start":191,"line_end":191,"column_start":44,"column_end":58,"is_primary":true,"text":[{"text":"            self.ring.len().saturating_sub(1 + self.len())","highlight_start":44,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:191:44\n    |\n191 |             self.ring.len().saturating_sub(1 + self.len())\n    |                                            ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5410,"byte_end":5484,"line_start":196,"line_end":196,"column_start":5,"column_end":79,"is_primary":true,"text":[{"text":"    fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":5,"highlight_end":79}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:196:5\n    |\n196 |     fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5906,"byte_end":5916,"line_start":216,"line_end":216,"column_start":16,"column_end":26,"is_primary":true,"text":[{"text":"        assert(buf@.1 > 1);","highlight_start":16,"highlight_end":26}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:216:16\n    |\n216 |         assert(buf@.1 > 1);\n    |                ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":5993,"byte_end":6010,"line_start":220,"line_end":220,"column_start":17,"column_end":34,"is_primary":true,"text":[{"text":"                buf@.0.len() == 0,","highlight_start":17,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:220:17\n    |\n220 |                 buf@.0.len() == 0,\n    |                 ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":6118,"byte_end":6129,"line_start":224,"line_end":224,"column_start":20,"column_end":31,"is_primary":true,"text":[{"text":"            assert(enqueue_res);","highlight_start":20,"highlight_end":31}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:224:20\n    |\n224 |             assert(enqueue_res);\n    |                    ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":6189,"byte_end":6201,"line_start":227,"line_end":227,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(buf_len == 1);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:227:20\n    |\n227 |             assert(buf_len == 1);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3","byte_start":6275,"byte_end":6287,"line_start":230,"line_end":230,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(has_elements);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_mx302f3:230:20\n    |\n230 |             assert(has_elements);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 20 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 20 previous errors\n\n"}
//
//
