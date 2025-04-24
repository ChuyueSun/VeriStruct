use vstd::prelude::*;

verus! {
    struct RingBuffer<T> {
        buffer: Vec<T>,
        head: usize,
        tail: usize,
    }

    impl<T: Copy> RingBuffer<T> {
        pub fn new(cap: usize) -> Self
        {
            let mut buffer = Vec::new();
            buffer.reserve(cap);

            RingBuffer {
                buffer,
                head: 0,
                tail: 0,
            }
        }

        pub fn push(&mut self, value: T) -> bool
        {
            if self.is_full() {
                return false;
            }

            if self.buffer.len() < self.buffer.capacity() {
                self.buffer.push(value);
            } else {
                self.buffer.set(self.tail, value);
            }

            self.tail = (self.tail + 1) % self.buffer.capacity();
            true
        }

        pub fn pop(&mut self) -> Option<T>
        {
            if self.is_empty() {
                return None;
            }

            let value = self.buffer[self.head];
            self.head = (self.head + 1) % self.buffer.capacity();

            Some(value)
        }

        pub fn is_empty(&self) -> bool
        {
            self.head == self.tail
        }

        pub fn is_full(&self) -> bool
        {
            self.head == ((self.tail + 1) % self.buffer.capacity())
        }
    }
}
