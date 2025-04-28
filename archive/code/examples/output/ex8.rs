impl<T: Copy> View for RingBuffer<T> {
    type V = Seq<T>;

    closed spec fn view(&self) -> Self::V {
        let len = if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.ring.len() - self.head + self.tail
        };

        Seq::new(len as nat, |i| {
            let index = (self.head + i) % self.ring.len() as int;
            self.ring[index]
        })
    }
}
