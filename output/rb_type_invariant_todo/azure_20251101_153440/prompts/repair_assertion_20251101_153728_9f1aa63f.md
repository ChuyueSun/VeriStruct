# Prompt

## Instruction
Your mission is to fix the assertion errors in test functions. To do that, do not change the test code. Change the code that is called by the test function.

Fix the assertion error in the given Rust code by adding postconditions to implementation functions. Specifically:

1. **Analyze each failing `assert(P)`** to determine which function's return value is being tested.
2. **Add postconditions** to that function establishing property `P`.
3. Add BIDIRECTIONAL specifications.
5. **Only modify implementation functions** - do not change the test code.
6. Do not remove existing `#[trigger]` annotations.

**Response Format:**
Provide only the modified Rust code—no explanations.


**Seq Knowledge**:
Seq<T> is a mathematical sequence type used in specifications:
- Building: Seq::empty(), seq![x, y, z], Seq::singleton(x)
- Length: s.len()
- Indexing: s[i] (0-based)
- Subrange: s.subrange(lo, hi) gives elements from index lo (inclusive) to hi (exclusive)
- Concatenation: s1 + s2
- Update: s.update(i, v) returns a new sequence with index i updated to value v
- Contains: s.contains(v) checks if v is in the sequence
- Push/pop: s.push(v), s.pop() (returns new sequence, doesn't modify original)
You can use forall or exists for properties over sequences.

The proof block looks like this:
```
proof {
    // your proof code here
    // assert(...)
    // LEMMA_FUNCTION(...)
    // ...
} // Added by AI
```
Note, please add the assertion directly for the `proof fn` function and DO NOT use proof block.
You can only use the proof block for the `fn` and `pub fn` functions.

The ghost variable looks like this:
```
let ghost ...; // Added by AI
```

Note, please DO NOT modify all other proof blocks that are not related to the error. Just leave them as they are.

IMPORTANT:
1. Don't change the anything in immutable function(s): test. Instead, consider adjusting the preconditions or postconditions of other functions or methods.
2. Don't delete existing non-buggy `#[trigger]`, `use` statements, main function.




# relevant vstd lib knowledge

## verification_plan

## 1. Analysis Summary

Current State:
- The code defines a struct RingBuffer<T> with #[verifier::type_invariant] and an “impl<T: Copy> View for RingBuffer<T>” block, indicating a View is required.
- Multiple TODO placeholders indicate missing invariants, requires/ensures, and proofs.
- The presence of "View" plus the need for function specs and proofs shows a comprehensive verification effort is needed.

Identified Missing Components:
- Formal View functions for the RingBuffer struct.
- Type invariant details inside inv(&self).
- Requires/ensures clauses for multiple methods (len, has_elements, is_full, new, enqueue, dequeue, available_len).
- Proof code for the indicated “TODO: add proof” blocks.

Dependencies:
- The RingBuffer’s correctness depends on the ring, head, and tail state relationships.
- The ring’s length and wrap-around behavior must be preserved by the invariants and the methods’ specs.
- The ring-based computations must be captured in the View function and refined as needed before specifying the function behavior.

## 2. Verification Plan

**Selected Workflow:** Full Sequence Workflow

**Justification:**
Since the code explicitly contains a “View” implementation (“impl<T: Copy> View for RingBuffer<T>”), it requires a complete solution with the computation of a View function. The RingBuffer is also a struct, so the optional inv_inference step applies. Additionally, the presence of "TODO: add proof" triggers proof_generation at the end.

**Execution Steps:**
1. view_inference
2. view_refinement
3. inv_inference
4. spec_inference
5. proof_generation

**Module Conditions:**
- inv_inference: Yes (RingBuffer<T> is a struct requiring a type invariant).
- proof_generation: Yes (The code contains several “TODO: add proof” markers).



## Exemplars

### Example 1

## Query
Failed split assertion
```
Line 22-22:
    assert(result.is_none());
Line 23-23:
    assert(result == None::<i32>);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Test assertion for Option return value - missing forward direction
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    spec fn view(&self) -> Seq<T> {
        self.items@
    }

    pub fn pop(&mut self) -> (result: Option<T>)
        ensures
            result.is_none() ==> self@ == old(self)@,
    {
        if self.items.len() == 0 {
            None
        } else {
            Some(self.items.pop())
        }
    }
}

fn test() {
    let mut s = Stack { items: Vec::new() };
    let result = s.pop();
    // Test assertions fail: can't prove result.is_none() from empty state!
    assert(result.is_none());
    assert(result == None::<i32>);
}

}
```



## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Add bidirectional postconditions (forward direction)
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    spec fn view(&self) -> Seq<T> {
        self.items@
    }

    pub fn pop(&mut self) -> (result: Option<T>)
        ensures
            // Reverse direction (was already present)
            result.is_none() ==> self@ == old(self)@,
            // Added by AI: Forward direction - critical for test assertions!
            old(self)@.len() == 0 ==> result.is_none(),
            old(self)@.len() > 0 ==> result.is_some(),
    {
        if self.items.len() == 0 {
            None
        } else {
            Some(self.items.pop())
        }
    }
}

fn test() {
    let mut s = Stack { items: Vec::new() };
    let result = s.pop();  // Now we can prove result.is_none() from empty state
    assert(result.is_none());
    assert(result == None::<i32>);
}

}
```



### Example 2

## Query
Failed split assertion
```
Line 30-30:
    assert(!has_data);
Line 31-31:
    assert(count == 0);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Test assertions about derived state after operations
struct Container {
    data: Vec<u32>,
}

impl Container {
    spec fn view(&self) -> Seq<u32> {
        self.data@
    }

    pub fn is_empty(&self) -> (result: bool)
        ensures
            result ==> self@.len() == 0,
    {
        self.data.len() == 0
    }

    pub fn count(&self) -> (result: usize)
    {
        self.data.len()
    }
}

fn test() {
    let c = Container { data: Vec::new() };
    let has_data = !c.is_empty();
    let count = c.count();
    // Test assertions fail: missing postconditions to connect state to results
    assert(!has_data);
    assert(count == 0);
}

}
```



## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Add complete bidirectional postconditions
struct Container {
    data: Vec<u32>,
}

impl Container {
    spec fn view(&self) -> Seq<u32> {
        self.data@
    }

    pub fn is_empty(&self) -> (result: bool)
        ensures
            // Added by AI: Bidirectional specification
            result <==> self@.len() == 0,  // Both directions in one!
    {
        self.data.len() == 0
    }

    pub fn count(&self) -> (result: usize)
        ensures
            result == self@.len(),  // Added by AI: state the relationship
    {
        self.data.len()
    }
}

fn test() {
    let c = Container { data: Vec::new() };
    let has_data = !c.is_empty();  // Can prove has_data == false
    let count = c.count();          // Can prove count == 0
    assert(!has_data);
    assert(count == 0);
}

}
```



### Example 3

## Query
Failed split assertion
```
Line 35-35:
    assert(result);
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Test assertion about boolean return that depends on precondition
fn check_bounds(v: &Vec<u32>, index: usize) -> (result: bool)
    ensures
        result ==> index < v.len(),
{
    index < v.len()
}

fn process(v: &Vec<u32>, index: usize) -> (success: bool)
    requires
        index < v.len(),
    ensures
        success == true,
{
    let _val = v[index];
    true
}

fn test() {
    let v = vec![1u32, 2, 3, 4, 5];
    let index = 2;

    if check_bounds(&v, index) {
        let result = process(&v, index);
        // Test assertion fails: can't connect check_bounds result to process success
        assert(result);
    }
}

}
```



## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Add postcondition establishing precondition for dependent call
fn check_bounds(v: &Vec<u32>, index: usize) -> (result: bool)
    ensures
        result ==> index < v.len(),
        result <==> index < v.len(),  // Added by AI: bidirectional
{
    index < v.len()
}

fn process(v: &Vec<u32>, index: usize) -> (success: bool)
    requires
        index < v.len(),
    ensures
        success == true,
{
    let _val = v[index];
    true
}

fn test() {
    let v = vec![1u32, 2, 3, 4, 5];
    let index = 2;

    if check_bounds(&v, index) {
        let result = process(&v, index);  // Can now prove precondition from check_bounds
        assert(result);
    }
}

}
```



## Query
Failed split assertion
```
Line 274-274:
    assert(!has_elements);
```

Code
```
#[allow(unused_imports)]
use vstd::prelude::*;
use vstd::assert_seqs_equal;

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
        ex_saturating_sub_spec(a as int, b as int) == (ret as int)
{
    a.saturating_sub(b)
}

pub open spec fn mod_auto_plus(n: int) -> bool
    recommends
        n > 0
{
    forall|x: int, y: int|
        {
            let z = (x % n) + (y % n);
            ((0 <= z && z < n && #[trigger] ((x + y) % n) == z)
                ||(n <= z && z < n + n && ((x + y) % n) == z - n))
        }
}

pub open spec fn mod_auto_minus(n: int) -> bool
    recommends
        n > 0
{
    forall|x: int, y: int|
        {
            let z = (x % n) - (y % n);
            ((0 <= z && z < n && #[trigger] ((x - y) % n) == z)
                ||(-n <= z && z < 0 && ((x - y) % n) == z + n))
        }
}

pub open spec fn mod_auto(n: int) -> bool
    recommends
        n > 0
{
    &&& (n % n == 0 && (-n) % n == 0)
    &&& (forall|x: int| #[trigger] ((x % n) % n) == x % n)
    &&& (forall|x: int| 0 <= x && x < n <==> #[trigger] (x % n) == x)
    &&& mod_auto_plus(n)
    &&& mod_auto_minus(n)
}

pub proof fn lemma_mod_auto(n: int)
    requires
        n > 0
    ensures
        mod_auto(n)
{
    admit()
}

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> View for RingBuffer<T> {
    type V = (Seq<T>, nat);

    closed spec fn view(&self) -> Self::V {
        let ring_view = self.ring@;
        let c = if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.ring.len() - self.head + self.tail
        };
        let content = Seq::new(c as nat, |i: int|
            ring_view[((self.head as int + i) % (ring_view.len() as int)) as int]
        );
        (content, ring_view.len() as nat)
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
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
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn len(&self) -> (ret: usize)
        ensures
            (ret as int) == self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
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
            ret == (self@.0.len() > 0)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
        ensures
            ret == (self@.0.len() == self@.1 - 1)
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring@.len() > 0
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring@.len()
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
        ensures
            self@.1 == old(self)@.1,
            succ ==> self@.0 =~= old(self)@.0 + seq![val],
            !succ ==> self@.0 == old(self)@.0
    {
        if self.is_full() {
            false
        } else {
            let old_tail = self.tail;
            my_set(&mut self.ring, old_tail, val);
            // Update tail before proving the ring's logical view has grown:
            self.tail = (self.tail + 1) % self.ring.len();
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                assert_seqs_equal!(self.ring@, old(self).ring@.update(old_tail as int, val));
                assert(self@.0 =~= old(self)@.0 + seq![val]);
            }
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            ret.is_some() ==> ret.unwrap() == old(self)@.0[0],
            ret.is_some() ==> self@.0 == old(self)@.0.subrange(1, (old(self)@.0.len()) as int),
            ret.is_none() ==> ret == None::<T>,
            ret.is_none() ==> self@.0 == old(self)@.0
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
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
            (ret as int) == self@.1 - 1 - self@.0.len()
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self.ring.len() as int);
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

#[verifier::loop_isolation(false)]
fn test(len: usize, value: i32, iterations: usize)
    requires
        1 < len < usize::MAX - 1,
        iterations * 2 < usize::MAX
{
    let mut ring: Vec<i32>= Vec::new();

    if len == 0 {
        return;
    }

    for i in 0..(len + 1)
    invariant
        ring.len() == i,
    {
        ring.push(0);
    }

    assert(ring.len() == len + 1);
    let mut buf = RingBuffer::new(ring);

    let ret = buf.dequeue();
    let buf_len = buf.len();
    let has_elements = buf.has_elements();
    assert(!has_elements);
    assert(ret == None::<i32>);
    assert(buf_len == 0);
    assert(len > 1);
    for i in 0..len
    invariant
        buf@.0.len() == i,
        buf@.1 == len + 1
    {
        let enqueue_res = buf.enqueue(value);
        assert(enqueue_res);
        let has_elements = buf.has_elements();
        assert(has_elements);
        let available_len = buf.available_len();
        assert(available_len == len - 1 - i);
    }
    let dequeue_res = buf.dequeue();
    assert(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    assert(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    assert(!enqueue_res);
    let dequeue_res = buf.dequeue();
    assert(dequeue_res.is_some());
}

pub fn main() {
}

}```
