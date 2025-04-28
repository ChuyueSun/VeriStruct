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
            let ring_spec = self.ring@;
            let capacity = ring_spec.len();
            let data = if self.tail >= self.head {
                ring_spec.subrange(self.head as int, self.tail as int)
            } else {
                ring_spec.subrange(self.head as int, ( capacity ) as int) + ring_spec.subrange(0, self.tail as int)
            };
            (data, capacity)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 17
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFailVecLen: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":2938,"byte_end":2943,"line_start":94,"line_end":94,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":3692,"byte_end":3722,"line_start":130,"line_end":130,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:130:13\n    |\n94  |             n > 0,\n    |             ----- failed precondition\n...\n130 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":3857,"byte_end":3886,"line_start":135,"line_end":135,"column_start":13,"column_end":42,"is_primary":true,"text":[{"text":"            (self.ring.len() - self.head) + self.tail","highlight_start":13,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:135:13\n    |\n135 |             (self.ring.len() - self.head) + self.tail\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":3857,"byte_end":3898,"line_start":135,"line_end":135,"column_start":13,"column_end":54,"is_primary":true,"text":[{"text":"            (self.ring.len() - self.head) + self.tail","highlight_start":13,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:135:13\n    |\n135 |             (self.ring.len() - self.head) + self.tail\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":2938,"byte_end":2943,"line_start":94,"line_end":94,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":4519,"byte_end":4549,"line_start":159,"line_end":159,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:159:13\n    |\n94  |             n > 0,\n    |             ----- failed precondition\n...\n159 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":4583,"byte_end":4598,"line_start":161,"line_end":161,"column_start":23,"column_end":38,"is_primary":true,"text":[{"text":"        self.head == ((self.tail + 1) % self.ring.len())","highlight_start":23,"highlight_end":38}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:161:23\n    |\n161 |         self.head == ((self.tail + 1) % self.ring.len())\n    |                       ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":2938,"byte_end":2943,"line_start":94,"line_end":94,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5259,"byte_end":5289,"line_start":186,"line_end":186,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:186:17\n    |\n94  |             n > 0,\n    |             ----- failed precondition\n...\n186 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":3121,"byte_end":3139,"line_start":105,"line_end":105,"column_start":9,"column_end":27,"is_primary":false,"text":[{"text":"        i < old(vec).len(),","highlight_start":9,"highlight_end":27}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5317,"byte_end":5355,"line_start":188,"line_end":188,"column_start":13,"column_end":51,"is_primary":true,"text":[{"text":"            my_set(&mut self.ring, self.tail, val);","highlight_start":13,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:188:13\n    |\n105 |         i < old(vec).len(),\n    |         ------------------ failed precondition\n...\n188 |             my_set(&mut self.ring, self.tail, val);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5381,"byte_end":5396,"line_start":189,"line_end":189,"column_start":25,"column_end":40,"is_primary":true,"text":[{"text":"            self.tail = (self.tail + 1) % self.ring.len();","highlight_start":25,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:189:25\n    |\n189 |             self.tail = (self.tail + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":2938,"byte_end":2943,"line_start":94,"line_end":94,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5671,"byte_end":5701,"line_start":200,"line_end":200,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:200:13\n    |\n94  |             n > 0,\n    |             ----- failed precondition\n...\n200 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/std_specs/vec.rs","byte_start":1740,"byte_end":1760,"line_start":52,"line_end":52,"column_start":9,"column_end":29,"is_primary":false,"text":[{"text":"        i < vec.view().len(),","highlight_start":9,"highlight_end":29}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5769,"byte_end":5789,"line_start":204,"line_end":204,"column_start":23,"column_end":43,"is_primary":true,"text":[{"text":"            let val = self.ring[self.head];","highlight_start":23,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:204:23\n    |\n204 |             let val = self.ring[self.head];\n    |                       ^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/std_specs/vec.rs:52:9\n    |\n52  |         i < vec.view().len(),\n    |         -------------------- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":5815,"byte_end":5830,"line_start":205,"line_end":205,"column_start":25,"column_end":40,"is_primary":true,"text":[{"text":"            self.head = (self.head + 1) % self.ring.len();","highlight_start":25,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:205:25\n    |\n205 |             self.head = (self.head + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6213,"byte_end":6227,"line_start":221,"line_end":221,"column_start":40,"column_end":54,"is_primary":true,"text":[{"text":"        self.ring.len().saturating_sub(1 + self.len())","highlight_start":40,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:221:40\n    |\n221 |         self.ring.len().saturating_sub(1 + self.len())\n    |                                        ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6273,"byte_end":6347,"line_start":226,"line_end":226,"column_start":1,"column_end":75,"is_primary":true,"text":[{"text":"fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":1,"highlight_end":75}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:226:1\n    |\n226 | fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6701,"byte_end":6711,"line_start":246,"line_end":246,"column_start":12,"column_end":22,"is_primary":true,"text":[{"text":"    assert(buf@.1 > 1);","highlight_start":12,"highlight_end":22}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:246:12\n    |\n246 |     assert(buf@.1 > 1);\n    |            ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6776,"byte_end":6793,"line_start":250,"line_end":250,"column_start":13,"column_end":30,"is_primary":true,"text":[{"text":"            buf@.0.len() == 0,","highlight_start":13,"highlight_end":30}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:250:13\n    |\n250 |             buf@.0.len() == 0,\n    |             ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6885,"byte_end":6896,"line_start":254,"line_end":254,"column_start":16,"column_end":27,"is_primary":true,"text":[{"text":"        assert(enqueue_res);","highlight_start":16,"highlight_end":27}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:254:16\n    |\n254 |         assert(enqueue_res);\n    |                ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":6948,"byte_end":6960,"line_start":257,"line_end":257,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(buf_len == 1);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:257:16\n    |\n257 |         assert(buf_len == 1);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws","byte_start":7026,"byte_end":7038,"line_start":260,"line_end":260,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(has_elements);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpf2sjzvws:260:16\n    |\n260 |         assert(has_elements);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 17 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 17 previous errors\n\n"}
//
//
