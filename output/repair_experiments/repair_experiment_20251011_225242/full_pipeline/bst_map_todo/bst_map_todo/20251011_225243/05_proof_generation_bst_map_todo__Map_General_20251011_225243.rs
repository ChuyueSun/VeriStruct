pub fn insert(&mut self, key: u64, value: V)
    ensures
        self.view() =~= old(self).view().insert(key, value),
{
    proof {
        // Begin by invoking the type invariant for self to allow Verus to use BST properties
        use_type_invariant(&*self);
        // The `insert_into_optional` call below, together with TreeMap's definition of view(),
        // establishes the postcondition that self.view() == old(self).view().insert(key, value).
    }

    let mut root = None;
    std::mem::swap(&mut root, &mut self.root);
    Node::<V>::insert_into_optional(&mut root, key, value);
    self.root = root;
}

// Step 5 (proof_generation) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1
