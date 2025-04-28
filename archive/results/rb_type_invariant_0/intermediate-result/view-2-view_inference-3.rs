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
            (self.ring@, ( self.head ) as nat, ( self.tail ) as nat)
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


// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 20
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.Other: possible division by zeroVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.Other: possible division by zeroVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFailVecLen: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.Other: possible division by zeroVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":2625,"byte_end":2630,"line_start":87,"line_end":87,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":3336,"byte_end":3366,"line_start":121,"line_end":121,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:121:13\n    |\n87  |             n > 0,\n    |             ----- failed precondition\n...\n121 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":3501,"byte_end":3530,"line_start":126,"line_end":126,"column_start":13,"column_end":42,"is_primary":true,"text":[{"text":"            (self.ring.len() - self.head) + self.tail","highlight_start":13,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:126:13\n    |\n126 |             (self.ring.len() - self.head) + self.tail\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":3501,"byte_end":3542,"line_start":126,"line_end":126,"column_start":13,"column_end":54,"is_primary":true,"text":[{"text":"            (self.ring.len() - self.head) + self.tail","highlight_start":13,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:126:13\n    |\n126 |             (self.ring.len() - self.head) + self.tail\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":2625,"byte_end":2630,"line_start":87,"line_end":87,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4079,"byte_end":4109,"line_start":148,"line_end":148,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:148:13\n    |\n87  |             n > 0,\n    |             ----- failed precondition\n...\n148 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4142,"byte_end":4177,"line_start":150,"line_end":150,"column_start":22,"column_end":57,"is_primary":true,"text":[{"text":"        self.head == ((self.tail + 1) % self.ring.len())","highlight_start":22,"highlight_end":57}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:150:22\n    |\n150 |         self.head == ((self.tail + 1) % self.ring.len())\n    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4143,"byte_end":4158,"line_start":150,"line_end":150,"column_start":23,"column_end":38,"is_primary":true,"text":[{"text":"        self.head == ((self.tail + 1) % self.ring.len())","highlight_start":23,"highlight_end":38}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:150:23\n    |\n150 |         self.head == ((self.tail + 1) % self.ring.len())\n    |                       ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":2625,"byte_end":2630,"line_start":87,"line_end":87,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4730,"byte_end":4760,"line_start":172,"line_end":172,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:172:17\n    |\n87  |             n > 0,\n    |             ----- failed precondition\n...\n172 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":2808,"byte_end":2826,"line_start":98,"line_end":98,"column_start":9,"column_end":27,"is_primary":false,"text":[{"text":"        i < old(vec).len(),","highlight_start":9,"highlight_end":27}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4788,"byte_end":4826,"line_start":174,"line_end":174,"column_start":13,"column_end":51,"is_primary":true,"text":[{"text":"            my_set(&mut self.ring, self.tail, val);","highlight_start":13,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:174:13\n    |\n98  |         i < old(vec).len(),\n    |         ------------------ failed precondition\n...\n174 |             my_set(&mut self.ring, self.tail, val);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4852,"byte_end":4867,"line_start":175,"line_end":175,"column_start":25,"column_end":40,"is_primary":true,"text":[{"text":"            self.tail = (self.tail + 1) % self.ring.len();","highlight_start":25,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:175:25\n    |\n175 |             self.tail = (self.tail + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":4852,"byte_end":4885,"line_start":175,"line_end":175,"column_start":25,"column_end":58,"is_primary":true,"text":[{"text":"            self.tail = (self.tail + 1) % self.ring.len();","highlight_start":25,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:175:25\n    |\n175 |             self.tail = (self.tail + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":2625,"byte_end":2630,"line_start":87,"line_end":87,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5100,"byte_end":5130,"line_start":185,"line_end":185,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:185:13\n    |\n87  |             n > 0,\n    |             ----- failed precondition\n...\n185 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/std_specs/vec.rs","byte_start":1740,"byte_end":1760,"line_start":52,"line_end":52,"column_start":9,"column_end":29,"is_primary":false,"text":[{"text":"        i < vec.view().len(),","highlight_start":9,"highlight_end":29}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5198,"byte_end":5218,"line_start":189,"line_end":189,"column_start":23,"column_end":43,"is_primary":true,"text":[{"text":"            let val = self.ring[self.head];","highlight_start":23,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:189:23\n    |\n189 |             let val = self.ring[self.head];\n    |                       ^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/std_specs/vec.rs:52:9\n    |\n52  |         i < vec.view().len(),\n    |         -------------------- failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5244,"byte_end":5259,"line_start":190,"line_end":190,"column_start":25,"column_end":40,"is_primary":true,"text":[{"text":"            self.head = (self.head + 1) % self.ring.len();","highlight_start":25,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:190:25\n    |\n190 |             self.head = (self.head + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible division by zero","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5244,"byte_end":5277,"line_start":190,"line_end":190,"column_start":25,"column_end":58,"is_primary":true,"text":[{"text":"            self.head = (self.head + 1) % self.ring.len();","highlight_start":25,"highlight_end":58}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible division by zero\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:190:25\n    |\n190 |             self.head = (self.head + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5594,"byte_end":5608,"line_start":203,"line_end":203,"column_start":40,"column_end":54,"is_primary":true,"text":[{"text":"        self.ring.len().saturating_sub(1 + self.len())","highlight_start":40,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:203:40\n    |\n203 |         self.ring.len().saturating_sub(1 + self.len())\n    |                                        ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":5654,"byte_end":5728,"line_start":208,"line_end":208,"column_start":1,"column_end":75,"is_primary":true,"text":[{"text":"fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":1,"highlight_end":75}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:208:1\n    |\n208 | fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":6082,"byte_end":6092,"line_start":228,"line_end":228,"column_start":12,"column_end":22,"is_primary":true,"text":[{"text":"    assert(buf@.1 > 1);","highlight_start":12,"highlight_end":22}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:228:12\n    |\n228 |     assert(buf@.1 > 1);\n    |            ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":6157,"byte_end":6174,"line_start":232,"line_end":232,"column_start":13,"column_end":30,"is_primary":true,"text":[{"text":"            buf@.0.len() == 0,","highlight_start":13,"highlight_end":30}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:232:13\n    |\n232 |             buf@.0.len() == 0,\n    |             ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":6266,"byte_end":6277,"line_start":236,"line_end":236,"column_start":16,"column_end":27,"is_primary":true,"text":[{"text":"        assert(enqueue_res);","highlight_start":16,"highlight_end":27}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:236:16\n    |\n236 |         assert(enqueue_res);\n    |                ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":6329,"byte_end":6341,"line_start":239,"line_end":239,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(buf_len == 1);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:239:16\n    |\n239 |         assert(buf_len == 1);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm","byte_start":6407,"byte_end":6419,"line_start":242,"line_end":242,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(has_elements);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpkg74fbpm:242:16\n    |\n242 |         assert(has_elements);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 20 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 20 previous errors\n\n"}
//
//
