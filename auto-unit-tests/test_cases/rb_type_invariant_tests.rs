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
    fn test_ex_saturating_sub_normal() {
        // When a >= b, the result is a - b.
        assert_eq!(ex_saturating_sub(10, 5), 5);
        // When a < b, subtracting should saturate to 0.
        assert_eq!(ex_saturating_sub(3, 5), 0);
    }

    #[test]
    fn test_ringbuffer_empty() {
        // Create a RingBuffer with a fixed capacity of 5.
        // Note: maximum storable elements is 4 because of the is_full condition.
        let rb = RingBuffer::<i32>::new(vec![0; 5]);
        // Initially empty.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 4); // 5 - 1 - 0 = 4
        assert!(!rb.has_elements());
        assert!(!rb.is_full());
    }

    #[test]
    fn test_ringbuffer_enqueue_and_dequeue() {
        let mut rb = RingBuffer::new(vec![0; 5]);
        // Enqueue one element.
        assert!(rb.enqueue(42));
        assert_eq!(rb.len(), 1);
        // Dequeue should return the enqueued element.
        assert_eq!(rb.dequeue(), Some(42));
        // Now the buffer is empty again.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 4);
    }

    #[test]
    fn test_ringbuffer_full() {
        let mut rb = RingBuffer::new(vec![0; 5]);
        // The effective capacity is 4 elements.
        assert!(rb.enqueue(1));
        assert!(rb.enqueue(2));
        assert!(rb.enqueue(3));
        assert!(rb.enqueue(4));

        // At this point, the buffer should be full.
        assert!(rb.is_full());
        assert_eq!(rb.available_len(), 0);
        // Trying to enqueue in a full buffer should fail.
        assert!(!rb.enqueue(5));

        // Dequeue one element and then we should be able to enqueue.
        assert_eq!(rb.dequeue(), Some(1));
        assert!(!rb.is_full());
        assert_eq!(rb.available_len(), 1);
        assert!(rb.enqueue(5));
    }

    #[test]
    fn test_ringbuffer_dequeue_empty() {
        let mut rb = RingBuffer::new(vec![0; 5]);
        // Dequeue on an empty buffer should return None.
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ringbuffer_wrap_around() {
        let mut rb = RingBuffer::new(vec![0; 5]);
        // Enqueue 3 elements.
        assert!(rb.enqueue(10));
        assert!(rb.enqueue(20));
        assert!(rb.enqueue(30));
        // Dequeue 2 elements.
        assert_eq!(rb.dequeue(), Some(10));
        assert_eq!(rb.dequeue(), Some(20));
        // Now head has moved and we have one element left.
        assert_eq!(rb.len(), 1);
        // At this point, available_len should be 5 - 1 - 1 = 3.
        assert_eq!(rb.available_len(), 3);
        // Enqueue 3 more elements to force tail wrap-around.
        assert!(rb.enqueue(40));
        assert!(rb.enqueue(50));
        assert!(rb.enqueue(60));  // This should succeed because the buffer now holds 4 elements total.
        // Now the buffer should be full.
        assert!(rb.is_full());
        // Dequeue all elements and check the order.
        // Order should be: leftover from first enqueue (30), then newly enqueued: 40, 50, 60.
        assert_eq!(rb.dequeue(), Some(30));
        assert_eq!(rb.dequeue(), Some(40));
        assert_eq!(rb.dequeue(), Some(50));
        assert_eq!(rb.dequeue(), Some(60));
        // The buffer should now be empty.
        assert_eq!(rb.len(), 0);
        assert!(!rb.has_elements());
    }
}