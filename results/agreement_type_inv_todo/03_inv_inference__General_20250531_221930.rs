#[verifier::type_invariant]
closed spec fn inv(&self) -> bool {
    self.r.value().valid()
}

// Step 3 (inv_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1