#[verifier(external_body)]
pub fn insert(&mut self, key: u64, value: V)
    requires
        true
    ensures
        self@ =~= old(self)@.insert(key, value)
{
    proof {
        use_type_invariant(&*self);
        // The call to insert_into_optional carries the ensures
        //   Node::<V>::optional_as_map(*root) =~= Node::<V>::optional_as_map(*old(root)).insert(key, value).
        // After swapping back, self.root = root => self@ = Node::<V>::optional_as_map(*root).
        // Hence self@ =~= old(self)@.insert(key, value).
    }
    let mut root = None;
    std::mem::swap(&mut root, &mut self.root);
    Node::<V>::insert_into_optional(&mut root, key, value);
    self.root = root;
}

// VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
