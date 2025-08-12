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
        // When a > b.
        assert_eq!(ex_saturating_sub(10, 5), 5);
        // When a == b.
        assert_eq!(ex_saturating_sub(10, 10), 0);
        // When a < b.
        assert_eq!(ex_saturating_sub(5, 10), 0);
    }

    #[test]
    fn test_dequeue_empty() {
        // Create a ring buffer of capacity 3 (vector of length 4)
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 4]);
        // Initially empty.
        assert!(!buf.has_elements());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.dequeue(), None);
    }

    #[test]
    fn test_enqueue_dequeue() {
        // Create a ring buffer with a vector of length 4 (capacity = 3)
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 4]);
        // Initially, available_len should be 3.
        assert_eq!(buf.available_len(), 3);

        // Enqueue three elements.
        assert!(buf.enqueue(10));
        assert!(buf.enqueue(20));
        assert!(buf.enqueue(30));

        // Now it should be full.
        assert!(buf.is_full());
        assert!(buf.has_elements());
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.available_len(), 0);

        // Enqueue should fail when full.
        assert!(!buf.enqueue(40));

        // Dequeue one element.
        assert_eq!(buf.dequeue(), Some(10));
        // Now available length increases.
        assert!(buf.available_len() > 0);

        // Enqueue now should succeed.
        assert!(buf.enqueue(40));

        // Dequeue the remaining elements in order.
        assert_eq!(buf.dequeue(), Some(20));
        assert_eq!(buf.dequeue(), Some(30));
        assert_eq!(buf.dequeue(), Some(40));
        // Finally, buffer becomes empty.
        assert_eq!(buf.dequeue(), None);
    }

    #[test]
    fn test_full_wraparound() {
        // Use a ring buffer with a vector of length 5, so capacity is 4.
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 5]);

        // Enqueue 4 elements.
        assert!(buf.enqueue(1));
        assert!(buf.enqueue(2));
        assert!(buf.enqueue(3));
        assert!(buf.enqueue(4)); // This should fill the buffer.
        assert!(buf.is_full());
        assert_eq!(buf.len(), 4);

        // Dequeue two elements.
        assert_eq!(buf.dequeue(), Some(1));
        assert_eq!(buf.dequeue(), Some(2));
        // Now the buffer should not be full.
        assert!(!buf.is_full());
        // The length should now be 2.
        assert_eq!(buf.len(), 2);
        // There should be space for 2 more elements.
        assert_eq!(buf.available_len(), 2);

        // Enqueue two more to test wraparound.
        assert!(buf.enqueue(5));
        assert!(buf.enqueue(6));
        // Now the buffer should be full again.
        assert!(buf.is_full());
        assert_eq!(buf.len(), 4);

        // Dequeue all remaining elements and check order.
        // The expected order is the remaining original elements then the wrap-around ones.
        let expected = [3, 4, 5, 6];
        for exp in &expected {
            assert_eq!(buf.dequeue(), Some(*exp));
        }
        // Buffer should now be empty.
        assert_eq!(buf.dequeue(), None);
    }
}