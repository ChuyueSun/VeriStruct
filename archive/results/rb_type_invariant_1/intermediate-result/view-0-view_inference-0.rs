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
    /// Invariant for the ring buffer.
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.ring@.len() > 1
        && self.head < self.ring@.len()
        && self.tail < self.ring@.len()
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 13
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ConstructorFailTypeInvariant: constructed value may fail to meet its declared type invariantVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.ArithmeticFlow: possible arithmetic underflow/overflowVerusErrorType.AssertFail: assertion failedVerusErrorType.InvFailFront: invariant not satisfied before loopVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failedVerusErrorType.AssertFail: assertion failed
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":3464,"byte_end":3494,"line_start":124,"line_end":124,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:124:13\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n124 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":4291,"byte_end":4321,"line_start":153,"line_end":153,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:153:13\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n153 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":4355,"byte_end":4370,"line_start":155,"line_end":155,"column_start":23,"column_end":38,"is_primary":true,"text":[{"text":"        self.head == ((self.tail + 1) % self.ring.len())","highlight_start":23,"highlight_end":38}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:155:23\n    |\n155 |         self.head == ((self.tail + 1) % self.ring.len())\n    |                       ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"constructed value may fail to meet its declared type invariant","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":3118,"byte_end":3139,"line_start":112,"line_end":112,"column_start":17,"column_end":38,"is_primary":false,"text":[{"text":"    closed spec fn inv(&self) -> bool {","highlight_start":17,"highlight_end":38}],"label":"type invariant declared here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":4578,"byte_end":4660,"line_start":162,"line_end":166,"column_start":9,"column_end":10,"is_primary":true,"text":[{"text":"        RingBuffer {","highlight_start":9,"highlight_end":21},{"text":"            head: 0,","highlight_start":1,"highlight_end":21},{"text":"            tail: 0,","highlight_start":1,"highlight_end":21},{"text":"            ring,","highlight_start":1,"highlight_end":18},{"text":"        }","highlight_start":1,"highlight_end":10}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: constructed value may fail to meet its declared type invariant\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:162:9\n    |\n112 |       closed spec fn inv(&self) -> bool {\n    |                   --------------------- type invariant declared here\n...\n162 | /         RingBuffer {\n163 | |             head: 0,\n164 | |             tail: 0,\n165 | |             ring,\n166 | |         }\n    | |_________^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":5031,"byte_end":5061,"line_start":180,"line_end":180,"column_start":17,"column_end":47,"is_primary":true,"text":[{"text":"                lemma_mod_auto(self@.1 as int);","highlight_start":17,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:180:17\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n180 |                 lemma_mod_auto(self@.1 as int);\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":2654,"byte_end":2659,"line_start":88,"line_end":88,"column_start":13,"column_end":18,"is_primary":false,"text":[{"text":"            n > 0,","highlight_start":13,"highlight_end":18}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":5443,"byte_end":5473,"line_start":194,"line_end":194,"column_start":13,"column_end":43,"is_primary":true,"text":[{"text":"            lemma_mod_auto(self@.1 as int);","highlight_start":13,"highlight_end":43}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:194:13\n    |\n88  |             n > 0,\n    |             ----- failed precondition\n...\n194 |             lemma_mod_auto(self@.1 as int);\n    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":5587,"byte_end":5602,"line_start":199,"line_end":199,"column_start":25,"column_end":40,"is_primary":true,"text":[{"text":"            self.head = (self.head + 1) % self.ring.len();","highlight_start":25,"highlight_end":40}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:199:25\n    |\n199 |             self.head = (self.head + 1) % self.ring.len();\n    |                         ^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"possible arithmetic underflow/overflow","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":5983,"byte_end":5997,"line_start":213,"line_end":213,"column_start":40,"column_end":54,"is_primary":true,"text":[{"text":"        self.ring.len().saturating_sub(1 + self.len())","highlight_start":40,"highlight_end":54}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: possible arithmetic underflow/overflow\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:213:40\n    |\n213 |         self.ring.len().saturating_sub(1 + self.len())\n    |                                        ^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6043,"byte_end":6117,"line_start":218,"line_end":218,"column_start":1,"column_end":75,"is_primary":true,"text":[{"text":"fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)","highlight_start":1,"highlight_end":75}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:218:1\n    |\n218 | fn test_enqueue_dequeue_generic(len: usize, value: i32, iterations: usize)\n    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6471,"byte_end":6481,"line_start":238,"line_end":238,"column_start":12,"column_end":22,"is_primary":true,"text":[{"text":"    assert(buf@.1 > 1);","highlight_start":12,"highlight_end":22}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:238:12\n    |\n238 |     assert(buf@.1 > 1);\n    |            ^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"invariant not satisfied before loop","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6546,"byte_end":6563,"line_start":242,"line_end":242,"column_start":13,"column_end":30,"is_primary":true,"text":[{"text":"            buf@.0.len() == 0,","highlight_start":13,"highlight_end":30}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: invariant not satisfied before loop\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:242:13\n    |\n242 |             buf@.0.len() == 0,\n    |             ^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6655,"byte_end":6666,"line_start":246,"line_end":246,"column_start":16,"column_end":27,"is_primary":true,"text":[{"text":"        assert(enqueue_res);","highlight_start":16,"highlight_end":27}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:246:16\n    |\n246 |         assert(enqueue_res);\n    |                ^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6718,"byte_end":6730,"line_start":249,"line_end":249,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(buf_len == 1);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:249:16\n    |\n249 |         assert(buf_len == 1);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"assertion failed","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z","byte_start":6796,"byte_end":6808,"line_start":252,"line_end":252,"column_start":16,"column_end":28,"is_primary":true,"text":[{"text":"        assert(has_elements);","highlight_start":16,"highlight_end":28}],"label":"assertion failed","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: assertion failed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpj4x5oe1z:252:16\n    |\n252 |         assert(has_elements);\n    |                ^^^^^^^^^^^^ assertion failed\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 13 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 13 previous errors\n\n"}
// 
// 