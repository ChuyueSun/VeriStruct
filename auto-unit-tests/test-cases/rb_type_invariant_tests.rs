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
        // a > b
        assert_eq!(ex_saturating_sub(10, 5), 5);
        // a == b
        assert_eq!(ex_saturating_sub(7, 7), 0);
        // a < b, saturates to 0
        assert_eq!(ex_saturating_sub(3, 10), 0);
    }

    #[test]
    fn test_ringbuffer_empty() {
        // Create a ring buffer with capacity for 5 elements (effective capacity = 4).
        let ring = vec![0; 5];
        let mut rb = RingBuffer::new(ring);
        // Initially, the ring buffer is empty.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 4);
        assert!(!rb.has_elements());
        // Dequeue from an empty ring should return None.
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_enqueue_dequeue() {
        let ring = vec![0; 5];
        let mut rb = RingBuffer::new(ring);
        // Effective capacity is ring.len() - 1 = 4.
        // Enqueue 4 elements.
        assert!(rb.enqueue(10));
        assert!(rb.enqueue(20));
        assert!(rb.enqueue(30));
        assert!(rb.enqueue(40));
        // Now the buffer should be full.
        assert!(rb.is_full());
        // available_len should be 0.
        assert_eq!(rb.available_len(), 0);
        // Attempt to enqueue one more element should fail.
        assert!(!rb.enqueue(50));
        // The length should be 4.
        assert_eq!(rb.len(), 4);
        // Dequeue elements and confirm FIFO order.
        assert_eq!(rb.dequeue(), Some(10));
        assert_eq!(rb.dequeue(), Some(20));
        assert_eq!(rb.dequeue(), Some(30));
        assert_eq!(rb.dequeue(), Some(40));
        // Now the ring should be empty.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 4);
        assert!(!rb.has_elements());
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_wrap_around() {
        let ring = vec![0; 5];
        let mut rb = RingBuffer::new(ring);
        // Enqueue 4 elements to fill the ring buffer.
        assert!(rb.enqueue(1));
        assert!(rb.enqueue(2));
        assert!(rb.enqueue(3));
        assert!(rb.enqueue(4));
        // Buffer is full now.
        assert!(rb.is_full());
        // Dequeue one element (should be 1).
        assert_eq!(rb.dequeue(), Some(1));
        // Now there's space for one more element.
        assert!(!rb.is_full());
        // Enqueue another element; this should go to the wrapped-around position.
        assert!(rb.enqueue(5));
        // Now, the buffer should contain 2, 3, 4, 5 in FIFO order.
        assert_eq!(rb.len(), 4);
        assert!(rb.is_full());
        // Dequeue remaining elements and check order.
        assert_eq!(rb.dequeue(), Some(2));
        assert_eq!(rb.dequeue(), Some(3));
        assert_eq!(rb.dequeue(), Some(4));
        assert_eq!(rb.dequeue(), Some(5));
        // The buffer is empty again.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 4);
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_available_len() {
        let ring = vec![0; 6]; // effective capacity = 5
        let mut rb = RingBuffer::new(ring);
        // Initially available_len should be len(ring) - 1
        assert_eq!(rb.available_len(), 5);
        // Enqueue two elements.
        assert!(rb.enqueue(100));
        assert!(rb.enqueue(200));
        // Now len() is 2, so available_len should be 6 - 1 - 2 = 3.
        assert_eq!(rb.len(), 2);
        assert_eq!(rb.available_len(), 3);
        // Dequeue one element.
        assert_eq!(rb.dequeue(), Some(100));
        // Now len() is 1; available_len should be 6 - 1 - 1 = 4.
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.available_len(), 4);
    }
}