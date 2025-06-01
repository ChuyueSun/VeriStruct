impl<T> Lock<T> {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        self.inv.constant() == (self.atomic.id(), self.cell.id())
    }
}

// Step 3 (inv_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Verified: -1, Errors: 999, Verus Errors: 2