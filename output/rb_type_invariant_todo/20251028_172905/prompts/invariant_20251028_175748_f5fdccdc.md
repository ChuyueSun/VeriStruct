# Prompt

## Instruction
Your mission is to fix the arithmetic underflow/overflow error for the following code.
Basically, for each variable involved in the expression `(self.ring.len() - self.head)' in line `(self.ring.len() - self.head) + self.tail' of the program, there are several general ways to fix the error:

0. Make sure the value of EVERY variable involved in this expression is specified as a loop invariant.
1. Add a bound for the whole expression `(self.ring.len() - self.head)' as a loop invariant or as an assert. This bound can be a constant value, or another expression whose bound has been specified through loop invariants or asserts.
2. Or, add BOTH a lower bound (i.e. x > ..., x >= ...) AND an upper bound (i.e., x < ..., x <= ...) as an assertion or a loop invariant if they are in a loop body for EACH variable involved in the expression (self.ring.len() - self.head). If the variable is a loop index variable, make sure that its lower bound (e.g., its initial value at the beginning of the loop) and upper bound (based on the loop-exit condition) are specified as loop invariants. You may use the loop index variable in the invariant.

Do not miss any variable in `(self.ring.len() - self.head)', and do NOT add bound information related to any other variables. Please do not change function post-conditions.
Response requirements:
Respond with the verus code only, do not include any explanation.
You should only add loop invariants, and you should NOT make any other changes to the program.

Hint for the upper bound:
1. For the lower/upper bound, you don't always need to find the exact or strict value. Your mission is to find a provable bound for Verus, which is usually based on the loop index, like `car <= CONSTANT * index`.
2. If the expression involves the loop index or is updated during each loop iteration, use the loop index variable as the upper or lower bound in the invariant instead of using the CONSTANT alone!
3. If there is a non-linear upper bound, you can use a constant to represent part of the expression (e.g., a * CONSTANT_RELATED_TO_b) to make it linear. However, ensure that at least one variable remains (DO NOT USE A CONSTANT TO REPLACE THE WHOLE NON-LINEAR). This approach makes it easier to prove.
4. You may use conditional loop invariants to specify the upper bound based on the loop index. For example, `i > 0 ==> x < 10 * i` means that if `i` is greater than 0, then `x` is less than 10 times `i`.


## Exemplars

### Example 1

## Query
use vstd::prelude::*;
fn main() {}
verus! {
fn myfun(x: i32, y: i32) -> (r: i32)
  requires
    x < 10000,
    y < 10000,
    0 < x,
    0 < y,
{
    let mut i: usize = 0;
    let mut z: i32 = 0;
    while i < 4
        invariant
            0 <= i,
            i <= 4,
    {
      z = x + y;
      i += 1;
    }
    z
}
}


## Answer
use vstd::prelude::*;
fn main() {}
verus! {
fn myfun(x: i32, y: i32) -> (r: i32)
  requires
    x < 10000,
    y < 10000,
    0 < x,
    0 < y,
{
    let mut i: usize = 0;
    let mut z: i32 = 0;
    while i < 4
        invariant
            x < 10000, // Added by AI
            y < 10000, // Added by AI
            0 < x, // Added by AI
            0 < y, // Added by AI
            0 <= i,
            i <= 4,
    {
      z = x + y;
      i += 1;
    }
    z
}
}


### Example 2

## Query
use vstd::prelude::*;
fn main() {}
verus! {
fn myfun( ) -> (r: i32)
{
    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    while i < 100
        invariant
          i <= 100,
          i >= 0,
    {
      sum = i * i;
    }
    sum
}
}


## Answer
use vstd::prelude::*;
fn main() {}
verus! {
fn myfun( ) -> (r: i32)
{
    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    while i < 100
        invariant
          i <= 100,
          i >= 0,
          i * i <= 100 * i, // Added by AI, using 100 * i as the upper bound
    {
      sum = i * i;
    }
    sum
}
}


## Query
Arithmetic underflow/overflow
```
Line 119-119:
            (self.ring.len() - self.head) + self.tail
```

Code
```
use vstd::prelude::*;

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
                || (n <= z && z < n + n && ((x + y) % n) == z - n))
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
                || (-n <= z && z < 0 && ((x - y) % n) == z + n))
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
        let ring_len = self.ring@.len();
        let elements = if self.tail >= self.head {
            self.ring@.subrange(self.head as int, self.tail as int)
        } else {
            self.ring@.subrange(self.head as int, ring_len as int)
                + self.ring@.subrange(0, self.tail as int)
        };
        (elements, (ring_len - 1) as nat)
    }
}

#[verifier::external_body]
fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < old(vec).len()
    ensures
        vec@ == old(vec)@.update(i as int, value),
        vec@.len() == old(vec).len(),
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
        ret == self@.0.len()
    {
        proof {
            assert(self.tail > self.head ==> self.tail - self.head == self@.0.len());
            assert(self.tail < self.head ==> (self.ring.len() - self.head) + self.tail == self@.0.len());
            assert(self.tail == self.head ==> self@.0.len() == 0);
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
            assert(self.head != self.tail ==> self@.0.len() > 0);
            assert(self.head == self.tail ==> self@.0.len() == 0);
        }
        self.head != self.tail
    }

    pub fn is_full(&self) -> (ret: bool)
    ensures
        ret == (self@.0.len() == self@.1)
    {
        proof {
            assert((self.head == ((self.tail + 1) % ( self.ring.len() ) as int)) ==> self@.0.len() == self@.1);
            assert((self.head != ((self.tail + 1) % ( self.ring.len() ) as int)) ==> self@.0.len() != self@.1);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
    requires
        ring.len() > 0
    ensures
        ret@.0.len() == 0,
        ret@.1 == ring@.len() - 1
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> (succ: bool)
    ensures
        if old(self)@.0.len() == old(self)@.1 {
            !succ && self@ == old(self)@
        } else {
            succ
            && self@.0 == old(self)@.0.push(val)
            && self@.1 == old(self)@.1
        }
    {
        if self.is_full() {
            false
        } else {
            proof {
                assert(self@.1 == old(self)@.1);
                assert(self@.0 == old(self)@.0.push(val));
            }
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> (ret: Option<T>)
    ensures
        if old(self)@.0.len() == 0 {
            ret == None::<T> && self@ == old(self)@
        } else {
            exists|front_val: T|
                ret == Some(front_val)
                && front_val == old(self)@.0[0]
                && self@.0
                   == old(self)@.0.subrange(1, ( old(self)@.0.len() ) as int)
                && self@.1 == old(self)@.1
        }
    {
        proof {
            // Added by AI
            assert(self.inv());
            assert(self.head < self.ring.len());
        } // Added by AI
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
        (ret as int)
            == if self@.0.len() <= self@.1 { self@.1 - self@.0.len() } else { 0 }
    {
        proof {
            // Removed the mode-mismatched assertion of self.len() vs self@.0.len().
            assert(self@.1 == (self.ring.len() - 1) as nat);
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
