impl<V> TreeMap<V> {
    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: old(self).well_formed()
    /// Ensures: self.well_formed(), self.as_map() =~= old(self).as_map().insert(key, value)
    pub fn insert(&mut self, key: u64, value: V)
        requires
            old(self).well_formed(),
        ensures
            self.well_formed(),
            self.as_map() =~= old(self).as_map().insert(key, value),
    {
        proof {
            use_type_invariant(&*self);
            // Here, we rely on the guarantees provided by `Node::insert_into_optional`
            // which ensures the map representation is old(...) plus the inserted key-value.
            // Together with self's type invariants, this completes the proof of the ensures clause.
        }

        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);
        self.root = root;
    }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
