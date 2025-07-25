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
        assert_eq!(ex_saturating_sub(5, 3), 2);
        assert_eq!(ex_saturating_sub(3, 5), 0);
        assert_eq!(ex_saturating_sub(0, 0), 0);
        assert_eq!(ex_saturating_sub(10, 10), 0);
    }

    #[test]
    fn test_new_ringbuffer() {
        let buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        assert_eq!(buf.head, 0);
        assert_eq!(buf.tail, 0);
        assert_eq!(buf.ring.len(), 3);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_len_empty() {
        let buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_len_tail_greater_than_head() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 5]);
        buf.tail = 3;
        buf.head = 1;
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_len_tail_less_than_head() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 5]);
        buf.tail = 1;
        buf.head = 3;
        assert_eq!(buf.len(), 3); // (5 - 3) + 1 = 3
    }

    #[test]
    fn test_has_elements() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        assert!(!buf.has_elements());
        buf.tail = 1;
        assert!(buf.has_elements());
        buf.head = 1;
        assert!(!buf.has_elements());
    }

    #[test]
    fn test_is_full_true() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        buf.tail = 1;
        buf.head = 2;
        // (tail + 1) % ring.len() = (1 + 1) % 3 = 2 == head
        assert!(buf.is_full());
    }

    #[test]
    fn test_is_full_false() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        buf.tail = 1;
        buf.head = 1;
        assert!(!buf.is_full());
        buf.head = 0;
        assert!(!buf.is_full());
    }

    #[test]
    fn test_enqueue_success() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        assert!(buf.enqueue(10));
        assert_eq!(buf.ring[0], 10);
        assert_eq!(buf.tail, 1);
    }

    #[test]
    fn test_enqueue_full() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        buf.head = 1;
        buf.tail = 0;
        // buffer is full if head == (tail + 1) % ring.len()
        assert!(buf.is_full());
        assert!(!buf.enqueue(42));
    }

    #[test]
    fn test_dequeue_some() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        buf.ring[0] = 5;
        buf.tail = 1;
        assert_eq!(buf.dequeue(), Some(5));
        assert_eq!(buf.head, 1);
    }

    #[test]
    fn test_dequeue_none() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 3]);
        assert_eq!(buf.dequeue(), None);
    }

    #[test]
    fn test_available_len_empty() {
        let buf: RingBuffer<i32> = RingBuffer::new(vec![0; 4]);
        // len() == 0, available_len = 4 - 1 - 0 = 3
        assert_eq!(buf.available_len(), 3);
    }

    #[test]
    fn test_available_len_nonempty() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 4]);
        buf.tail = 2;
        buf.head = 0;
        // len() = tail - head = 2 - 0 = 2
        // available_len = 4 - 1 - 2 = 1
        assert_eq!(buf.available_len(), 1);
    }

    #[test]
    fn test_available_len_wraparound() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(vec![0; 4]);
        buf.head = 3;
        buf.tail = 1;
        // len = (4 - 3) + 1 = 2
        // available_len = 4 - 1 - 2 = 1
        assert_eq!(buf.available_len(), 1);
    }
}