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
        pub closed spec fn inv(&self) -> bool {
            if self.ring@.len() == 0 {
                self.head == 0 && self.tail == 0
            } else {
                self.head < self.ring@.len()
                && self.tail < self.ring@.len()
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

        pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            (old(self)@.0.len() == old(self)@.1 - 1) ==> (!succ && self@.0.len() == old(self)@.0.len()),
            (old(self)@.0.len() < old(self)@.1 - 1) ==> (succ && self@.0.len() == old(self)@.0.len() + 1),
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
            (old(self)@.0.len() > 0) ==> (ret.is_Some() && self@.0.len() == old(self)@.0.len() - 1),
            (old(self)@.0.len() == 0) ==> (ret.is_None() && self@.0.len() == old(self)@.0.len()),
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
            ret == ((self@.1 - 1) - self@.0.len()),
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
                buf@.1 > 1,
                0 <= buf.tail,
                buf.tail < buf.ring.len(),
                0 < buf.ring.len(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 11
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PostCondFail: postcondition not satisfiedVerusErrorType.InvFailEnd: invariant not satisfied at end of loop bodyVerusErrorType.InvFailEnd: invariant not satisfied at end of loop bodyVerusErrorType.InvFailEnd: invariant not satisfied at end of loop bodyVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":2210,"byte_end":2215,"line_start":83,"line_end":83,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":3125,"byte_end":3155,"line_start":119,"line_end":119,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:119:17\n    |\n83  |             n > 0,\n    |             ----- failed precondition\n...\n119 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":2210,"byte_end":2215,"line_start":83,"line_end":83,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":3856,"byte_end":3886,"line_start":146,"line_end":146,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:146:17\n    |\n83  |             n > 0,\n    |             ----- failed precondition\n...\n146 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":3928,"byte_end":3943,"line_start":148,"line_end":148,"column_start":27,"column_end":42,"is_primary":true,"text":[{"text":"            self.head == ((self.tail + 1) % self.ring.len())","highlight_start":27,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:148:27\n    |\n148 |             self.head == ((self.tail + 1) % self.ring.len())\n    |                           ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":2210,"byte_end":2215,"line_start":83,"line_end":83,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":4731,"byte_end":4761,"line_start":174,"line_end":174,"column_start":21,"column_end":51,"is_primary":true,"text":[{"text":"                    lemma_mod_auto(self@.1 as int);","highlight_start":21,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:174:21\n    |\n83  |             n > 0,\n    |             ----- failed precondition\n...\n174 |                     lemma_mod_auto(self@.1 as int);\n    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":2210,"byte_end":2215,"line_start":83,"line_end":83,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":5305,"byte_end":5335,"line_start":189,"line_end":189,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:189:17\n    |\n83  |             n > 0,\n    |             ----- failed precondition\n...\n189 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":5465,"byte_end":5480,"line_start":194,"line_end":194,"column_start":29,"column_end":44,"is_primary":true,"text":[{"text":"                self.head = (self.head + 1) % self.ring.len();","highlight_start":29,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:194:29\n    |\n194 |                 self.head = (self.head + 1) % self.ring.len();\n    |                             ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"postcondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":5812,"byte_end":5858,"line_start":208,"line_end":208,"column_start":13,"column_end":59,"is_primary":false,"text":[{"text":"            self.ring.len().saturating_sub(1 + self.len())","highlight_start":13,"highlight_end":59}],"label":"at the end of the function body","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":5673,"byte_end":5711,"line_start":203,"line_end":203,"column_start":13,"column_end":51,"is_primary":true,"text":[{"text":"            ret == ((self@.1 - 1) - self@.0.len()),","highlight_start":13,"highlight_end":51}],"label":"failed this postcondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: postcondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:203:13\n    |\n203 |             ret == ((self@.1 - 1) - self@.0.len()),\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ failed this postcondition\n...\n208 |             self.ring.len().saturating_sub(1 + self.len())\n    |             ---------------------------------------------- at the end of the function body\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied at end of loop body","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":6537,"byte_end":6547,"line_start":238,"line_end":238,"column_start":17,"column_end":27,"is_primary":true,"text":[{"text":"                buf@.1 > 1,","highlight_start":17,"highlight_end":27}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied at end of loop body\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:238:17\n    |\n238 |                 buf@.1 > 1,\n    |                 ^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied at end of loop body","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":6596,"byte_end":6621,"line_start":240,"line_end":240,"column_start":17,"column_end":42,"is_primary":true,"text":[{"text":"                buf.tail < buf.ring.len(),","highlight_start":17,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied at end of loop body\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:240:17\n    |\n240 |                 buf.tail < buf.ring.len(),\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied at end of loop body","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":6639,"byte_end":6657,"line_start":241,"line_end":241,"column_start":17,"column_end":35,"is_primary":true,"text":[{"text":"                0 < buf.ring.len(),","highlight_start":17,"highlight_end":35}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied at end of loop body\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:241:17\n    |\n241 |                 0 < buf.ring.len(),\n    |                 ^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j","byte_start":6975,"byte_end":7002,"line_start":253,"line_end":253,"column_start":20,"column_end":47,"is_primary":true,"text":[{"text":"            assert(dequeue_res =~= Some(value));","highlight_start":20,"highlight_end":47}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsi3x1d5j:253:20\n    |\n253 |             assert(dequeue_res =~= Some(value));\n    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 11 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 11 previous errors\n\n"}
// 
// 