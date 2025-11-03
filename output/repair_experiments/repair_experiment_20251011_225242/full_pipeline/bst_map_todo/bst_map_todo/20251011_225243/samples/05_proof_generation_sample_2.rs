#[verifier::external_body]
impl<V> TreeMap<V> {
    /// Inserts a key-value pair into the TreeMap.
    ///
    /// Requires: Nothing
    /// Ensures: self.view() is old(self).view() with (key, value) inserted
    pub fn insert(&mut self, key: u64, value: V)
        ensures
            self.view() =~= old(self).view().insert(key, value),
    {
        proof {
            use_type_invariant(&*self);
            // By definition: old(self)@ = Node::<V>::optional_as_map(old(self).root)
        }

        let mut root = None;
        std::mem::swap(&mut root, &mut self.root);
        Node::<V>::insert_into_optional(&mut root, key, value);

        proof {
            // From insert_into_optional, we know:
            // Node::<V>::optional_as_map(root) =~=
            //    Node::<V>::optional_as_map(old(root)).insert(key, value)
            // but old(root) is old(self).root, so:
            // Node::<V>::optional_as_map(root) =~=
            //    Node::<V>::optional_as_map(old(self).root).insert(key, value)
        }

        self.root = root;

        proof {
            // Now self@ = Node::<V>::optional_as_map(self.root),
            // which equals old(self).view().insert(key, value).
            assert(self@ =~= old(self)@.insert(key, value));
        }
    }
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
