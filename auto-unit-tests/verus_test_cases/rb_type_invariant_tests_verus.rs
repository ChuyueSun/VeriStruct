use vstd::prelude::*;
use vstd::vec::Vec;

verus! {

pub fn ex_saturating_sub(a: usize, b: usize) -> usize
    ensures
        result == if a >= b { a - b } else { 0 },
{
    a.saturating_sub(b)
}

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T)
    requires
        i < vec.len(),
{
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    pub fn len(&self) -> usize
        requires
            self.ring.len() > 0,
        ensures
            result == if self.tail >= self.head {
                        self.tail - self.head
                     } else {
                        (self.ring.len() - self.head) + self.tail
                     },
    {
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    pub fn has_elements(&self) -> bool
        requires
            self.ring.len() > 0,
        ensures
            result == (self.head != self.tail),
    {
        self.head != self.tail
    }

    pub fn is_full(&self) -> bool
        requires
            self.ring.len() > 0,
        ensures
            result == (self.head == ((self.tail + 1) % self.ring.len())),
    {
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> RingBuffer<T>
        requires
            ring.len() > 0,
        ensures
            result.head == 0,
            result.tail == 0,
            result.ring == ring,
    {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> bool
        requires
            self.ring.len() > 0,
        ensures
            if !result {
                self.head == old(self).head &&
                self.tail == old(self).tail &&
                self.ring == old(self).ring
            } else {
                self.tail == (old(self).tail + 1) % self.ring.len()
            },
    {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> Option<T>
        requires
            self.ring.len() > 0,
        ensures
            match result {
                Some(_) => {
                    self.head == (old(self).head + 1) % self.ring.len()
                },
                None => {
                    self.head == old(self).head &&
                    self.tail == old(self).tail
                },
            },
    {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    pub fn available_len(&self) -> usize
        requires
            self.ring.len() > 0,
        ensures
            result == self.ring.len().saturating_sub(1 + self.len()),
    {
        self.ring.len().saturating_sub(1 + self.len())
    }
}

fn create_ringbuffer_with_capacity<T: Copy + Default>(size: usize) -> RingBuffer<T>
    requires
        size > 0,
    ensures
        result.head == 0,
        result.tail == 0,
        result.ring.len() == size,
{
    let mut ring: Vec<T> = Vec::with_capacity(size);
    let mut i: usize = 0;
    while i < size
        invariant [
            i <= size,
            ring.len() == i,
        ]
        decreases (size - i)
    {
        ring.push(T::default());
        i = i + 1;
    }
    RingBuffer::new(ring)
}

/* TEST CODE BELOW */

pub fn main() {
    // Test ex_saturating_sub
    assert(ex_saturating_sub(10, 5) == 5);
    assert(ex_saturating_sub(5, 5) == 0);
    assert(ex_saturating_sub(5, 10) == 0);
    assert(ex_saturating_sub(0, 3) == 0);
    assert(ex_saturating_sub(usize::MAX, 0) == usize::MAX);

    // test_ringbuffer_empty
    {
        let capacity: usize = 4;
        let rb = create_ringbuffer_with_capacity::<i32>(capacity);
        assert(rb.len() == 0);
        assert(!rb.has_elements());
        assert(rb.available_len() == capacity - 1);
        assert(!rb.is_full());
    }

    // test_ringbuffer_enqueue_dequeue
    {
        let capacity: usize = 4;
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);
        assert(rb.enqueue(10));
        assert(rb.len() == 1);
        assert(rb.has_elements());
        assert(rb.available_len() == capacity - 1 - 1);

        let value = rb.dequeue();
        assert(value == Some(10));
        assert(rb.len() == 0);
        assert(!rb.has_elements());
        assert(rb.dequeue() == None);
    }

    // test_ringbuffer_full
    {
        let capacity: usize = 4;
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);
        assert(rb.enqueue(1));
        assert(rb.enqueue(2));
        assert(rb.enqueue(3));
        assert(rb.is_full());
        assert(rb.available_len() == 0);
        assert(!rb.enqueue(4));

        assert(rb.dequeue() == Some(1));
        assert(rb.dequeue() == Some(2));
        assert(rb.dequeue() == Some(3));
        assert(rb.dequeue() == None);
    }

    // test_ringbuffer_wrap_around
    {
        let capacity: usize = 4;
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);
        assert(rb.enqueue(10));
        assert(rb.enqueue(20));
        assert(rb.enqueue(30));
        assert(rb.is_full());
        assert(rb.len() == 3);

        let dequeued = rb.dequeue();
        assert(dequeued == Some(10));
        assert(!rb.is_full());
        assert(rb.enqueue(40));
        assert(rb.is_full());
        assert(rb.len() == 3);

        assert(rb.dequeue() == Some(20));
        assert(rb.dequeue() == Some(30));
        assert(rb.dequeue() == Some(40));
        assert(rb.dequeue() == None);
    }

    // test_ringbuffer_partial_usage
    {
        let capacity: usize = 5;
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);
        assert(rb.enqueue(100));
        assert(rb.enqueue(200));
        assert(rb.len() == 2);
        assert(!rb.is_full());
        assert(rb.has_elements());
        assert(rb.available_len() == capacity - 1 - 2);

        assert(rb.dequeue() == Some(100));
        assert(rb.len() == 1);
        assert(rb.enqueue(300));
        assert(rb.enqueue(400));
        assert(rb.len() == 3);
        assert(rb.available_len() == capacity - 1 - 3);

        assert(rb.dequeue() == Some(200));
        assert(rb.dequeue() == Some(300));
        assert(rb.dequeue() == Some(400));
        assert(rb.dequeue() == None);
    }
}
}