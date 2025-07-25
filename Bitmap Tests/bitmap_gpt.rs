pub fn main() {}

pub fn ex_saturating_sub(a: usize, b: usize) -> usize {
    a.saturating_sub(b)
}

pub struct RingBuffer<T: Copy> {
    ring: Vec<T>,
    head: usize,
    tail: usize,
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
            self.ring[self.tail] = val;
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
        assert_eq!(ex_saturating_sub(10, 5), 5);
        assert_eq!(ex_saturating_sub(5, 10), 0);
        assert_eq!(ex_saturating_sub(0, 0), 0);
    }

    #[test]
    fn test_new_ringbuffer() {
        let buf = RingBuffer::new(vec![0; 5]);
        assert_eq!(buf.len(), 0);
        assert!(!buf.has_elements());
        assert!(!buf.is_full());
    }

    #[test]
    fn test_enqueue_and_len_wraparound() {
        let mut buf = RingBuffer::new(vec![0; 3]);
        assert!(buf.enqueue(1));
        assert!(buf.enqueue(2));
        assert_eq!(buf.len(), 2);
        assert!(!buf.enqueue(3)); // full
        assert!(buf.is_full());
    }

    #[test]
    fn test_dequeue_order_and_wraparound() {
        let mut buf = RingBuffer::new(vec![0; 3]);
        assert!(buf.enqueue(1));
        assert!(buf.enqueue(2));
        assert_eq!(buf.dequeue(), Some(1));
        assert!(buf.enqueue(3)); // wraparound
        assert_eq!(buf.dequeue(), Some(2));
        assert_eq!(buf.dequeue(), Some(3));
        assert_eq!(buf.dequeue(), None);
    }

    #[test]
    fn test_available_len() {
        let mut buf = RingBuffer::new(vec![0; 4]);
        assert_eq!(buf.available_len(), 3);
        buf.enqueue(1);
        buf.enqueue(2);
        assert_eq!(buf.available_len(), 1);
        buf.dequeue();
        assert_eq!(buf.available_len(), 2);
    }

    #[test]
    fn test_full_then_empty_then_reuse() {
        let mut buf = RingBuffer::new(vec![0; 3]);
        assert!(buf.enqueue(10));
        assert!(buf.enqueue(20));
        assert!(!buf.enqueue(30));
        assert_eq!(buf.dequeue(), Some(10));
        assert!(buf.enqueue(30));
        assert_eq!(buf.dequeue(), Some(20));
        assert_eq!(buf.dequeue(), Some(30));
        assert_eq!(buf.dequeue(), None);
    }
}