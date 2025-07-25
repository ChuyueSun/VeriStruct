pub fn ex_saturating_sub(a: usize, b: usize) -> usize {
    a.saturating_sub(b)
}

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
}

fn my_set<T: Copy>(vec: &mut Vec<T>, i: usize, value: T) {
    vec[i] = value;
}

impl<T: Copy> RingBuffer<T> {
    pub fn len(&self) -> usize {
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    pub fn has_elements(&self) -> bool {
        self.head != self.tail
    }

    pub fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }

    pub fn new(ring: Vec<T>) -> RingBuffer<T> {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    pub fn enqueue(&mut self, val: T) -> bool {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    pub fn available_len(&self) -> usize {
        self.ring.len().saturating_sub(1 + self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ex_saturating_sub() {
        // Case: a > b
        assert_eq!(ex_saturating_sub(10, 5), 5);
        // Case: a == b
        assert_eq!(ex_saturating_sub(5, 5), 0);
        // Case: a < b (should saturate at 0)
        assert_eq!(ex_saturating_sub(5, 10), 0);
        // Edge: a is zero
        assert_eq!(ex_saturating_sub(0, 3), 0);
        // Edge: large number subtraction with b == 0
        assert_eq!(ex_saturating_sub(usize::MAX, 0), usize::MAX);
    }

    // Helper function to create a RingBuffer with a pre-allocated vector filled with a default value.
    fn create_ringbuffer_with_capacity<T: Copy + Default>(size: usize) -> RingBuffer<T> {
        // The underlying ring vector should have a fixed size.
        let ring = vec![T::default(); size];
        RingBuffer::new(ring)
    }

    #[test]
    fn test_ringbuffer_empty() {
        let capacity = 4;
        // For testing, use i32 as T.
        let rb = create_ringbuffer_with_capacity::<i32>(capacity);
        // Initially, buffer should be empty.
        assert_eq!(rb.len(), 0);
        assert!(!rb.has_elements());
        // available_len should be capacity - 1 (since one slot remains unused).
        assert_eq!(rb.available_len(), capacity - 1);
        // Not full as it has no elements.
        assert!(!rb.is_full());
    }

    #[test]
    fn test_ringbuffer_enqueue_dequeue() {
        let capacity = 4;
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);

        // Enqueue one element.
        assert!(rb.enqueue(10));
        assert_eq!(rb.len(), 1);
        assert!(rb.has_elements());
        // available_len should decrease.
        assert_eq!(rb.available_len(), capacity - 1 - 1);

        // Dequeue the element.
        let value = rb.dequeue();
        assert_eq!(value, Some(10));
        // Now buffer should be empty again.
        assert_eq!(rb.len(), 0);
        assert!(!rb.has_elements());

        // Dequeue from empty buffer should return None.
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_full() {
        let capacity = 4; // For a ring buffer of capacity 4, maximum elements = 3.
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);

        // Enqueue maximum allowed elements.
        assert!(rb.enqueue(1));
        assert!(rb.enqueue(2));
        assert!(rb.enqueue(3));

        // Buffer should now be full.
        assert!(rb.is_full());
        // available_len() should be zero.
        assert_eq!(rb.available_len(), 0);
        // Trying to enqueue another element should fail.
        assert!(!rb.enqueue(4));

        // Check ordering via dequeue.
        assert_eq!(rb.dequeue(), Some(1));
        assert_eq!(rb.dequeue(), Some(2));
        assert_eq!(rb.dequeue(), Some(3));
        // Now buffer should be empty.
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_wrap_around() {
        let capacity = 4; // Maximum elements = 3.
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);

        // Enqueue three elements to fill the buffer.
        assert!(rb.enqueue(10));
        assert!(rb.enqueue(20));
        assert!(rb.enqueue(30));
        // Buffer is full at this point.
        assert!(rb.is_full());
        assert_eq!(rb.len(), 3);

        // Dequeue one element to create space.
        let dequeued = rb.dequeue();
        assert_eq!(dequeued, Some(10));
        // Now buffer is not full.
        assert!(!rb.is_full());
        // Enqueue another element; this should go to the wrapped-around slot.
        assert!(rb.enqueue(40));
        // Now buffer should be full again.
        assert!(rb.is_full());
        assert_eq!(rb.len(), 3);

        // Dequeue remaining elements to test proper order.
        assert_eq!(rb.dequeue(), Some(20));
        assert_eq!(rb.dequeue(), Some(30));
        assert_eq!(rb.dequeue(), Some(40));
        // Finally, the buffer should be empty.
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_partial_usage() {
        let capacity = 5; // Maximum elements = 4.
        let mut rb = create_ringbuffer_with_capacity::<i32>(capacity);

        // Enqueue two elements.
        assert!(rb.enqueue(100));
        assert!(rb.enqueue(200));
        // Buffer properties.
        assert_eq!(rb.len(), 2);
        assert!(!rb.is_full());
        assert!(rb.has_elements());
        assert_eq!(rb.available_len(), capacity - 1 - 2);

        // Dequeue one element.
        assert_eq!(rb.dequeue(), Some(100));
        // After dequeue, length should be 1.
        assert_eq!(rb.len(), 1);
        // Enqueue two more elements.
        assert!(rb.enqueue(300));
        assert!(rb.enqueue(400));
        assert_eq!(rb.len(), 3);
        // available space.
        assert_eq!(rb.available_len(), capacity - 1 - 3);

        // Dequeue the rest and check order.
        assert_eq!(rb.dequeue(), Some(200));
        assert_eq!(rb.dequeue(), Some(300));
        assert_eq!(rb.dequeue(), Some(400));
        // Now empty.
        assert_eq!(rb.dequeue(), None);
    }
}