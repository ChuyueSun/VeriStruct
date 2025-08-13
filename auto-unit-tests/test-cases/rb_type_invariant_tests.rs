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
    /// Returns how many elements are in the buffer.
    pub fn len(&self) -> usize {
        if self.tail > self.head {
            self.tail - self.head
        } else if self.tail < self.head {
            (self.ring.len() - self.head) + self.tail
        } else {
            0
        }
    }

    /// Returns true if there are any items in the buffer, false otherwise.
    pub fn has_elements(&self) -> bool {
        self.head != self.tail
    }

    /// Returns true if the buffer is full, false otherwise.
    pub fn is_full(&self) -> bool {
        self.head == ((self.tail + 1) % self.ring.len())
    }

    /// Creates a new RingBuffer with the given backing `ring` storage.
    pub fn new(ring: Vec<T>) -> RingBuffer<T> {
        RingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// This method attempts to add a new element to the back of the ring buffer.
    /// The operation succeeds only if the buffer is not full.
    /// 
    /// # Arguments
    /// * `val` - The value to add to the buffer
    /// 
    /// # Returns
    /// * `true` - If the element was successfully added (buffer was not full)
    /// * `false` - If the element could not be added (buffer was full)
    /// 
    /// # Invariants
    /// * The ring buffer's capacity remains unchanged
    /// * If successful, the length increases by 1 and the new value is at the end
    /// * If unsuccessful, the buffer remains unchanged
    /// * All previously enqueued elements remain in their original positions
    pub fn enqueue(&mut self, val: T) -> bool {
        if self.is_full() {
            false
        } else {
            my_set(&mut self.ring, self.tail, val);
            self.tail = (self.tail + 1) % self.ring.len();
            true
        }
    }

    /// Removes and returns the front element from the ring buffer, if one exists.
    /// 
    /// This method attempts to remove and return the oldest element (front) from the buffer.
    /// If the buffer is empty, it returns None.
    /// 
    /// # Returns
    /// * `Some(T)` - The front element if the buffer was not empty
    /// * `None` - If the buffer was empty
    /// 
    /// # Invariants
    /// * The ring buffer's capacity remains unchanged
    /// * If an element is returned, the buffer's length decreases by 1
    /// * If an element is returned, all remaining elements shift forward one position
    /// * If no element is returned (empty buffer), the buffer remains unchanged
    pub fn dequeue(&mut self) -> Option<T> {
        if self.has_elements() {
            let val = self.ring[self.head];
            self.head = (self.head + 1) % self.ring.len();
            Some(val)
        } else {
            None
        }
    }

    /// Returns the number of elements that can still be enqueued until it is full.
    pub fn available_len(&self) -> usize {
        self.ring.len().saturating_sub(1 + self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ex_saturating_sub_no_underflow() {
        // When a >= b, simple subtraction occurs.
        assert_eq!(ex_saturating_sub(10, 5), 5);
        assert_eq!(ex_saturating_sub(5, 5), 0);
    }

    #[test]
    fn test_ex_saturating_sub_underflow() {
        // When a < b, saturating_sub should yield 0.
        assert_eq!(ex_saturating_sub(3, 5), 0);
        assert_eq!(ex_saturating_sub(0, 1), 0);
    }

    fn create_buffer_with_capacity<T: Copy>(capacity: usize, init_val: T) -> RingBuffer<T> {
        // Create a vector of given capacity
        let vec = vec![init_val; capacity];
        RingBuffer::new(vec)
    }

    #[test]
    fn test_ring_buffer_new_empty() {
        let rb: RingBuffer<i32> = create_buffer_with_capacity(5, 0);
        // Initially the buffer is empty.
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.available_len(), 5 - 1);  // because one slot is reserved
        assert!(!rb.has_elements());
        // When buffer is empty, dequeue should return None.
        let mut rb = rb; // mutable copy for dequeue test
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ring_buffer_enqueue_dequeue() {
        let mut rb: RingBuffer<i32> = create_buffer_with_capacity(5, 0);
        // Enqueue a single element
        assert!(rb.enqueue(42));
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.available_len(), 5 - 1 - 1);
        assert!(rb.has_elements());

        // Dequeue that element
        let val = rb.dequeue();
        assert_eq!(val, Some(42));
        assert_eq!(rb.len(), 0);
        assert!(!rb.has_elements());
    }

    #[test]
    fn test_ring_buffer_full_behavior() {
        let mut rb: RingBuffer<i32> = create_buffer_with_capacity(4, 0);
        // With capacity vector length 4, maximum allowed elements is 3.
        assert!(rb.enqueue(1));
        assert!(rb.enqueue(2));
        assert!(rb.enqueue(3));
        // Buffer should now be full.
        assert!(rb.is_full());
        assert_eq!(rb.len(), 3);
        // Attempting to enqueue another element should fail.
        assert!(!rb.enqueue(4));
        // The length remains unchanged.
        assert_eq!(rb.len(), 3);

        // Dequeue one element which should be 1.
        let val = rb.dequeue();
        assert_eq!(val, Some(1));
        // Buffer is no longer full.
        assert!(!rb.is_full());
        // Now enqueue should succeed.
        assert!(rb.enqueue(4));
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let mut rb: RingBuffer<i32> = create_buffer_with_capacity(5, 0);
        // Maximum number of elements is 4 (ring.len() - 1)
        // Enqueue 3 elements.
        assert!(rb.enqueue(10));
        assert!(rb.enqueue(20));
        assert!(rb.enqueue(30));
        // Dequeue two elements to move the head.
        assert_eq!(rb.dequeue(), Some(10));
        assert_eq!(rb.dequeue(), Some(20));
        // Now enqueue two more elements; these should go to the beginning of the ring.
        assert!(rb.enqueue(40));
        assert!(rb.enqueue(50));
        // At this point the ordering should be: 30, 40, 50
        assert_eq!(rb.len(), 3);
        // Dequeue remaining elements to verify order.
        assert_eq!(rb.dequeue(), Some(30));
        assert_eq!(rb.dequeue(), Some(40));
        assert_eq!(rb.dequeue(), Some(50));
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_ring_buffer_available_len_calculation() {
        let mut rb: RingBuffer<i32> = create_buffer_with_capacity(6, 0);
        // With a backing storage of 6, maximum elements = 5.
        // Initially available length should be 5.
        assert_eq!(rb.available_len(), 5);

        // Enqueue three elements.
        assert!(rb.enqueue(1));
        assert!(rb.enqueue(2));
        assert!(rb.enqueue(3));
        // Now available length should be 6 - 1 - 3 = 2.
        assert_eq!(rb.available_len(), 2);

        // Dequeue one element, then available increases.
        assert_eq!(rb.dequeue(), Some(1));
        // Now available length should be 6 - 1 - 2 = 3.
        assert_eq!(rb.available_len(), 3);
    }
}