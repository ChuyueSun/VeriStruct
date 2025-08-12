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

    pub fn ring_len(&self) -> usize {
        self.ring.len()
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

/* TEST CODE BELOW */

fn test(len: usize, value: i32, iterations: usize) {
    let mut ring: Vec<i32> = Vec::new();

    if len == 0 {
        return;
    }

    for _i in 0..(len + 1) {
        ring.push(0);
    }

    assert!(ring.len() == len + 1);
    let mut buf = RingBuffer::new(ring);

    let ret = buf.dequeue();
    let buf_len = buf.len();
    let has_elements = buf.has_elements();
    assert!(!has_elements);
    assert!(ret == None);
    assert!(buf_len == 0);
    assert!(len > 1);
    for i in 0..len {
        let enqueue_res = buf.enqueue(value);
        assert!(enqueue_res);
        let has_elements = buf.has_elements();
        assert!(has_elements);
        let available_len = buf.available_len();
        assert!(available_len == len - 1 - i);
    }
    let dequeue_res = buf.dequeue();
    assert!(dequeue_res.is_some());
    let enqueue_res = buf.enqueue(value);
    assert!(enqueue_res);
    let enqueue_res = buf.enqueue(value);
    assert!(!enqueue_res);
    let dequeue_res = buf.dequeue();
    assert!(dequeue_res.is_some());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_test() {
        test(5, 42, 1);
    }
}

pub fn main() {
}