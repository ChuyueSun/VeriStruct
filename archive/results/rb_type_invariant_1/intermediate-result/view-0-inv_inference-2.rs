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
            let content =
                if self.tail >= self.head {
                    ring_seq.subrange(self.head as int, self.tail as int)
                } else {
                    ring_seq.subrange(self.head as int, ( capacity ) as int)
                        + ring_seq.subrange(0, self.tail as int)
                };
            (content, capacity)
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
            self.ring.len() > 0
            && self.head < self.ring.len()
            && self.tail < self.ring.len()
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 7
// VerusErrorType.ConstructorFailTypeInvariant: constructed value may fail to meet its declared type invariantVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"constructed value may fail to meet its declared type invariant","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":2689,"byte_end":2710,"line_start":104,"line_end":104,"column_start":21,"column_end":42,"is_primary":false,"text":[{"text":"        closed spec fn inv(&self) -> bool {","highlight_start":21,"highlight_end":42}],"label":"type invariant declared here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":3778,"byte_end":3876,"line_start":144,"line_end":148,"column_start":13,"column_end":14,"is_primary":true,"text":[{"text":"            RingBuffer {","highlight_start":13,"highlight_end":25},{"text":"                head: 0,","highlight_start":1,"highlight_end":25},{"text":"                tail: 0,","highlight_start":1,"highlight_end":25},{"text":"                ring,","highlight_start":1,"highlight_end":22},{"text":"            }","highlight_start":1,"highlight_end":14}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: constructed value may fail to meet its declared type invariant\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:144:13\n    |\n104 |           closed spec fn inv(&self) -> bool {\n    |                       --------------------- type invariant declared here\n...\n144 | /             RingBuffer {\n145 | |                 head: 0,\n146 | |                 tail: 0,\n147 | |                 ring,\n148 | |             }\n    | |_____________^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":4952,"byte_end":4966,"line_start":187,"line_end":187,"column_start":44,"column_end":58,"is_primary":true,"text":[{"text":"            self.ring.len().saturating_sub(1 + self.len())","highlight_start":44,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:187:44\n    |\n187 |             self.ring.len().saturating_sub(1 + self.len())\n    |                                            ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5028,"byte_end":5102,"line_start":192,"line_end":192,"column_start":5,"column_end":79,"is_primary":true,"text":[{"text":"    fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":5,"highlight_end":79}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:192:5\n    |\n192 |     fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5524,"byte_end":5534,"line_start":212,"line_end":212,"column_start":16,"column_end":26,"is_primary":true,"text":[{"text":"        assert(buf@.1 > 1);","highlight_start":16,"highlight_end":26}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:212:16\n    |\n212 |         assert(buf@.1 > 1);\n    |                ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5611,"byte_end":5628,"line_start":216,"line_end":216,"column_start":17,"column_end":34,"is_primary":true,"text":[{"text":"                buf@.0.len() == 0,","highlight_start":17,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:216:17\n    |\n216 |                 buf@.0.len() == 0,\n    |                 ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5736,"byte_end":5747,"line_start":220,"line_end":220,"column_start":20,"column_end":31,"is_primary":true,"text":[{"text":"            assert(enqueue_res);","highlight_start":20,"highlight_end":31}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:220:20\n    |\n220 |             assert(enqueue_res);\n    |                    ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5807,"byte_end":5819,"line_start":223,"line_end":223,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(buf_len == 1);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:223:20\n    |\n223 |             assert(buf_len == 1);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e","byte_start":5893,"byte_end":5905,"line_start":226,"line_end":226,"column_start":20,"column_end":32,"is_primary":true,"text":[{"text":"            assert(has_elements);","highlight_start":20,"highlight_end":32}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdfm9ww0e:226:20\n    |\n226 |             assert(has_elements);\n    |                    ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 7 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 7 previous errors\n\n"}
// 
// 