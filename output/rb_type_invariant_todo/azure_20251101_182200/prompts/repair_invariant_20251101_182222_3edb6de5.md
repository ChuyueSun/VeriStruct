# Prompt

## Instruction
Your mission is to fix the constructor so that its declared type invariant is satisfied.
Often, this means adding `requires` to the constructor function so that the `inv(&self) -> bool` function is true.

**IMPORTANT**:
- DO NOT add `ret.inv()` to the ensures clause - type invariants are automatically checked by Verus!

Respond with the **fixed Rust code only** and do not include any explanation.

# Verus Common Knowledge

## Important Notes
- ALWAYS use parentheses whenever possible for clarity!
- Don't delete existing non-buggy `#[trigger]`!
- Don't change "unwind" to `(unwind) as bool`!
- Return the complete modified Rust code in your response without explanations.
- Keep top level docstrings at the top of the file, before `verus! {`. Do not place them after the `verus! {` declaration.
- Don't change any function signatures.

## Spec Functions
1. No Direct Method Calls:
   In a spec function, you cannot directly call instance methods such as vector.is_full().
2. Use the @ Operator:
   To invoke methods on a variable within a spec, first convert it to its specification-level representation View with @.
3. Always use vector.len() instead of vector@.len().
4. Simplify Boolean Conjunctions:
   When combining multiple conditions, avoid excessive &&&. Fewer (or well-structured) conjunctions make the spec code easier to read and debug.
5. Parentheses Usage:
   ALWAYS wrap conditions in parentheses, even for simple expressions. This makes precedence explicit and prevents errors.

## Proof Blocks - CRITICAL SYNTAX RULES

**🚫 NEVER use executable control flow (if/else/match) inside `proof { }` blocks!**

Proof blocks are spec-level contexts. They can only contain:
- `assert(...)` statements
- `assume(...)` statements
- Lemma/proof function calls
- Variable bindings with spec expressions

❌ **WRONG - Executable if/else in proof:**
```rust
proof {
    if condition { assert(x); } else { assert(y); }  // SYNTAX ERROR!
}
```

✅ **CORRECT - Use implication instead:**
```rust
proof {
    assert(condition ==> x);
    assert(!condition ==> y);
}
```

❌ **WRONG - Executable match in proof:**
```rust
proof {
    match opt { Some(v) => assert(v > 0), None => {} }  // SYNTAX ERROR!
}
```

✅ **CORRECT - Use implication or spec-level reasoning:**
```rust
proof {
    assert(opt.is_Some() ==> opt.unwrap() > 0);
}
```

## Operators
Verus extends Rust logical operators with low-precedence forms that are especially helpful in specification code:

Standard Operators: &&, ||, ==>, <==>
Low-Precedence Variants: &&& and |||

The meaning of &&& is the same as && (logical AND), and ||| is the same as || (logical OR), but with lower precedence. This allows you to write conditions in a "bulleted list" style that remains grouped in a logical manner:

```
&&& a ==> b
&&& c
&&& d <==> e && f
```

is equivalent to:

```
(a ==> b) && c && (d <==> (e && f))
```

Note:
- Implication (==>) and equivalence (<==>) bind more tightly than &&& and |||.
- Using &&&/||| can make long specifications clearer by grouping logical clauses neatly.


# Verus Sequence Knowledge

Seq<T> is a mathematical sequence type used in specifications:

- Building: Seq::empty(), seq![x, y, z], Seq::singleton(x)
- Length: s.len()
- Indexing: s[i] (0-based)
- Subrange: s.subrange(lo, hi) gives elements from index lo (inclusive) to hi (exclusive)
- Concatenation: s1 + s2
- Update: s.update(i, v) returns a new sequence with index i updated to value v
- Contains: s.contains(v) checks if v is in the sequence
- Push/pop: s.push(v), s.pop() (returns new sequence, doesn't modify original)
- Equality: Use `=~=` for sequence equality, NOT `==` (e.g., `seq1 =~= seq2`, `self@.0 =~= old(self)@.0.subrange(1, (old(self)@.0.len()) as int)`)

You can use forall or exists for properties over sequences.

## CRITICAL: Postconditions for Seq<T> Update Operations

**For functions that update a single element in a sequence-based view**:

**✅ PREFER** - Use `.update()` for succinct, provable specifications:
```rust
fn update_element(&mut self, idx: usize, value: T)
    requires
        idx < old(self)@.len(),
    ensures
        self@ == old(self)@.update(idx as int, value),  // ✅ Succinct!
```

**❌ AVOID** - Verbose element-wise specifications (makes proofs much harder):
```rust
ensures
    self@.len() == old(self)@.len(),
    forall|i: int| 0 <= i && i < self@.len() && i != idx as int ==> self@[i] == old(self)@[i],
    self@[idx as int] == value,
```

**Why `.update()` is better**:
1. More concise and readable
2. Directly matches proof patterns (pairs with `assert_seqs_equal!`)
3. Easier for Verus SMT solver to reason about
4. Standard pattern in Verus for sequence modifications

**When to use this pattern**:
- Any function that modifies exactly one position in a Seq-based view
- After operations that update a single element (e.g., `self.data.set(index, value)`)
- Functions with postconditions about changing one element while preserving others

## Subrange Usage

1. Cast indices to `int`: `self.head as int`, not just `self.head`
2. Use `.subrange(start, end)`, not `seq[start..end]`
3. Wrap complex expressions: `(..).subrange((expression) as int, ...)`
4. For full length: `(sequence.len()) as int`


additional knowledge:
IMPORTANT:
1. Don't change the anything in immutable function(s): test. Instead, consider adjusting the preconditions or postconditions of other functions or methods.
2. Don't delete existing non-buggy `#[trigger]`, `use` statements, main function.


## Exemplars

### Example 1

## Query
Failed assertion
```
Line 32-32:
            helper(&mut self.buffer, self.index);
Error: constructed value may fail to meet its declared type invariant
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Type invariant reconstruction after mutable field borrow
fn helper<T>(vec: &mut Vec<T>, index: usize)
    ensures
        vec@.len() == old(vec)@.len(),
{}

struct Container<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> Container<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.buffer.len() > 0
        &&& self.index < self.buffer.len()  // Depends on buffer.len()!
    }

    pub fn modify(&mut self)
    {
        // Error: Verus can't prove type invariant holds after helper returns
        // because index < buffer.len() depends on buffer.len() being unchanged
        helper(&mut self.buffer, self.index);
    }
}

}
```



## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Prove type invariant holds before borrowing field
fn helper<T>(vec: &mut Vec<T>, index: usize)
    ensures
        vec@.len() == old(vec)@.len(),
{}

struct Container<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> Container<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.buffer.len() > 0
        &&& self.index < self.buffer.len()  // Depends on buffer.len()!
    }

    pub fn modify(&mut self)
    {
        proof {
            use_type_invariant(&*self); // Added by AI: get current invariant
            // Added by AI: Assert properties that must hold after helper returns
            // Added by AI: Since helper preserves buffer.len(), these still hold:
            assert(self.index < self.buffer.len());
        }
        helper(&mut self.buffer, self.index);
    }
}

}
```



### Example 2

## Query
Failed assertion
```
Line 45-45:
            my_set(&mut self.ring, self.tail, val);
Error: constructed value may fail to meet its declared type invariant
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Ring buffer with type invariant - field borrow in method
#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
{
    vec[i] = value;
}

struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn enqueue(&mut self, val: T)
    {
        if !self.is_full() {
            // Error: After my_set returns, Verus must verify:
            // - self.head < self.ring.len()
            // - self.tail < self.ring.len()
            // But it can't prove these automatically!
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
        }
    }

    spec fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }
}

}
```



```
Line 45-45:
            my_set(&mut self.ring, self.tail, val);
Error: constructed value may fail to meet its declared type invariant
```

Code
```
use vstd::prelude::*;
fn main() {}

verus! {

// Pattern: Ring buffer with type invariant - field borrow in method
#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
{
    vec[i] = value;
}

struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn enqueue(&mut self, val: T)
    {
        if !self.is_full() {
            // Error: After my_set returns, Verus must verify:
            // - self.head < self.ring.len()
            // - self.tail < self.ring.len()
            // But it can't prove these automatically!
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
        }
    }

    spec fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }
}

}
```




## Answer
use vstd::prelude::*;
fn main() {}

verus! {

// Fix: Use type_invariant before borrowing to prove reconstruction will succeed
#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
{
    vec[i] = value;
}

struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn enqueue(&mut self, val: T)
    {
        if !self.is_full() {
            proof {
                use_type_invariant(&*self); // Added by AI: get current invariant
                // Added by AI: my_set preserves vec.len(), so these will still hold:
                assert(self.head < self.ring.len());
                assert(self.tail < self.ring.len());
            }
            my_set(&mut self.ring, self.tail, val); // ✓ Reconstruction succeeds
            self.tail = (self.tail + 1) % self.ring.len();
        }
    }

    spec fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }
}

}



fn main() {}

verus! {

// Fix: Use type_invariant before borrowing to prove reconstruction will succeed
#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len(),
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
{
    vec[i] = value;
}

struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy> RingBuffer<T> {
    #[verifier::type_invariant]
    spec fn inv(&self) -> bool {
        &&& self.ring.len() > 0
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
    }

    pub fn enqueue(&mut self, val: T)
    {
        if !self.is_full() {
            proof {
                use_type_invariant(&*self); // Added by AI: get current invariant
                // Added by AI: my_set preserves vec.len(), so these will still hold:
                assert(self.head < self.ring.len());
                assert(self.tail < self.ring.len());
            }
            my_set(&mut self.ring, self.tail, val); // ✓ Reconstruction succeeds
            self.tail = (self.tail + 1) % self.ring.len();
        }
    }

    spec fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }
}

}




## Query
In constructor, the declared type invariant is not satisfied:
```
    closed spec fn inv(&self) -> bool {
```

Code
```
/// This file demonstrates a ring buffer in Verus.
/// It includes specification functions for modular operations as well as
/// a partially-specified ring buffer data structure.
///
/// Fill in the missing specification pieces (invariants, requires, ensures, proofs)
/// as needed to verify correctness.

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

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the sum `x % n + y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x + y) % n`.
    /// (2) It's in the range `[n, 2n)` and equals `(x + y) % n + n`.
    pub open spec fn mod_auto_plus(n: int) -> bool
        recommends
            n > 0
    {
        forall|x: int, y: int|
            {
                let z = (x % n) + (y % n);
                ((0 <= z && z < n && #[trigger] ((x + y) % n) == z)
                    ||(n <= z && z < n + n&& ((x + y) % n) == z - n))
            }
    }

    /// This function says that for any `x` and `y`, there are two
    /// possibilities for the difference `x % n - y % n`:
    /// (1) It's in the range `[0, n)` and equals `(x - y) % n`.
    /// (2) It's in the range `[-n, 0)` and equals `(x - y) % n - n`.
    pub open spec fn mod_auto_minus(n: int) -> bool
        recommends
            n > 0
    {
        forall|x: int, y: int|
            {
                let z = (x % n) - (y % n);
                ((0 <= z && z < n && #[trigger] ((x - y) % n) == z)
                    ||(-n <= z && z < 0&& ((x - y) % n) == z + n))
            }
    }

    /// This function states various useful properties about the modulo
    /// operator when the divisor is `n`.
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

    /// Proof of `mod_auto(n)`, which states various useful properties
    /// about the modulo operator when the divisor is the positive
    /// number `n`
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
                ring_view[((self.head as int + i) % ( ring_view.len() ) as int) as int]
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
            // The code below exactly matches the ring content length logic.
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
            // has elements <=> head != tail <=> content length > 0
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
            // is full <=> head == (tail + 1) % ring.len()
            // <=> content length == capacity - 1
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() > 0
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
            my_set(&mut self.ring, self.tail, val);
            proof {
                use_type_invariant(&*self);
                lemma_mod_auto(self.ring.len() as int);
                assert_seqs_equal!(self.ring@, old(self).ring@.update(self.tail as int, val));
                assert(self@.0 =~= old(self)@.0 + seq![val]);
            }
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
        ensures
            self@.1 == old(self)@.1,
            ret.is_some() ==> ret.unwrap() == old(self)@.0[0],
            ret.is_some() ==> self@.0 == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int),
            ret.is_none() ==> ret == None::<T>,
            ret.is_none() ==> self@.0 == old(self)@.0
    {
        proof {
            use_type_invariant(&*self);
            lemma_mod_auto(self.ring.len() as int);
            // If has elements, we're removing one item: new content is a subrange of old content.
            // Otherwise, no change.
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
            // available_len = ring.len() - 1 - content length
        }
        self.ring.len().saturating_sub(1 + self.len())
    }
}

/* TEST CODE BELOW */

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

} // verus!```


Additional Context:


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
