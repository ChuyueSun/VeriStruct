impl<V> DoublyLinkedList<V> {
    /// Representation of this list as a sequence
    pub closed spec fn view(&self) -> Seq<V> {
        Seq::<V>::new(
            self.ghost_state@.ptrs.len(),
            |i: int| { self.ghost_state@.points_to_map[i as nat].value().payload },
        )
    }
}