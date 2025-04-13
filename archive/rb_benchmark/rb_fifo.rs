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
        type V = (Seq<T>, usize);
    
        closed spec fn view(&self) -> Self::V {
            let cap = self.ring.len();
            if self.tail >= self.head {
                ((self.ring)@.subrange(self.head as int, self.tail as int),
                cap)
            } else {
                ((self.ring)@.subrange(self.head as int, cap as int)
                    .add((self.ring)@.subrange(0, self.tail as int)),
                    cap)
            }
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
        &&& self.head < self.ring.len()
        &&& self.tail < self.ring.len()
        &&& self.ring.len() > 0
    }


    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> (ret: usize)
        ensures
            ret == self@.0.len()
    {
        proof {
            use_type_invariant(&self);
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
        ensures
            ret == (self@.0.len() != 0)
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
    ensures
        ret == (self@.0.len() == (self@.1 - 1) as nat) 
    {
        proof {
            use_type_invariant(&self);
            lemma_mod_auto(self@.1 as int);
        }
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> (ret: RingBuffer<T>)
        requires
            ring.len() >= 1
        ensures
            ret@.0.len() == 0,
            ret@.1 == ring.len()
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
        ensures
            // Full fails iff old(len) == capacity => !succ
            old(self)@.0.len() == (old(self)@.1 - 1) as nat <==> !succ,
            // The ring size itself doesn't change:
            self@.1 == old(self)@.1,
            // If succ, length increments by 1:
            succ == (self@.0.len() == old(self)@.0.len() + 1),
            // The newly enqueued value is at the end:
            succ ==> (self@.0.last() == val),
            // Previous elements unchanged:
            forall |i: int|
                0 <= i < old(self)@.0.len() ==> self@.0[i] == old(self)@.0[i]
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
        ensures
            // The ring size remains unchanged
            self@.1 == old(self)@.1,
            // Empty fails
            old(self)@.0.len() == 0 <==> ret == None::<T>,
            old(self)@.0.len() > 0 <==> ret != None::<T>,
            
            if let Some(val) = ret {
                &&& self@.0.len() == old(self)@.0.len() - 1
                &&& val == old(self)@.0.first()
                &&& forall |i: int| 0 <= i < old(self)@.0.len() - 1 ==> self@.0[i] == old(self)@.0[i+1]
            } else {
                &&& self@.0.len() == old(self)@.0.len()
                &&& forall |i: int| 0 <= i < old(self)@.0.len() ==> self@.0[i] == old(self)@.0[i]
            }
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
    ensures ret == self@.1 - self@.0.len() - 1
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








#[verifier::loop_isolation(false)]
fn test_fifo_property() {
    let mut ring: Vec<i32> = Vec::new();


    for i in 0..(4)
        invariant
            ring.len() == i,
    {
        ring.push(0);
    }

    assert(ring.len() > 1);
    let mut buf = RingBuffer::new(ring);
    

    let er = buf.enqueue(1);
    buf.enqueue(2);
    buf.enqueue(3);
    // assert(buf@ == s123());

    let d1 = buf.dequeue();
    assert(d1 =~= Some(1));
    // assert(buf@ == s23());

    let d2 = buf.dequeue();
    assert(d2 =~= Some(2));

    buf.enqueue(4);
    // assert(buf@ == s34());
    
    let d3 = buf.dequeue();
    assert(d3 =~= Some(3));   
}






fn get_next_value(current: i32) -> (res: i32)
    ensures res != current
{
    current.wrapping_mul(1664525).wrapping_add(1013904223)
}


#[verifier::exec]
fn should_enqueue(x: i32) -> (res: bool)
    ensures res == true || res == false
{
    get_next_value(x) & 1 == 0
}


#[verifier::loop_isolation(false)]
fn test_fifo_property_generic(size: i32, iterations: i32, seed: i32)
    requires
        1 <= size < i32::MAX - 1,
        iterations < i32::MAX - 1, 
        1 <= iterations <= size - 1
{
    let mut ring: Vec<i32> = Vec::new();
    let mut reference_queue: Vec<i32> = Vec::new(); // Parallel structure for correctness

    for i in 0..(size + 1)
        invariant
            ring.len() == i,
    {
        ring.push(0);
    }

    let mut buf = RingBuffer::new(ring);

    // buf@.0.len() == buf@.1 - 1 == (size + 1) - 1 == size means the buffer is full


    let mut x: i32 = seed;

    let mut l: i32 = 0;
    for i in 0..iterations
    invariant
        buf@.1 == size + 1,
        buf@.0.len() == l,
        reference_queue.len() == l,
        l <= (buf@.1 - 1) as nat,
        forall |i: int| 0 <= i < l ==> buf@.0[i] == reference_queue[i]        
    {
        x = get_next_value(x);
        
        if should_enqueue(x) || reference_queue.len() == 0 {
            let enqueue_res = buf.enqueue(x);
            
            if enqueue_res {
                reference_queue.push(x);
                l = l + 1;
            } else {
                break;
            }
        }
        else {
            let dequeue_res = buf.dequeue();
            let expected_value = reference_queue.remove(0);             
            assert(dequeue_res =~= Some(expected_value));
            l = l - 1;            
        }

    }
    
}




}