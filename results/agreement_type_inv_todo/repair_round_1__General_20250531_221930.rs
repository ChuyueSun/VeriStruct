#[verifier::type_invariant]
spec fn inv(&self) -> bool {
    self.r.value().valid()
}

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1