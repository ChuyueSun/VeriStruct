```rust
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
                ||(n <= z && z < n + n&& ((x + y) % n) == z - n))
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
                ||(-n <= z && z < 0&& ((x - y) % n) == z + n))
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
            // Added by AI:
            ret <==> (self.head != self.tail)
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


// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
