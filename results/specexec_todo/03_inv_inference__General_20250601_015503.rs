use vstd::prelude::*;

verus! {
pub struct Concrete {
    pub a: u64,
}

pub struct Abstract {
    pub a: nat,
}

impl View for Concrete {
    type V = Abstract;

    open spec fn view(&self) -> <Self as vstd::string::View>::V {
        let t = (self.a as nat,);
        Abstract {
            a: t.0,
        }
    }
}

impl Concrete {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        true
    }

    #[verifier::when_used_as_spec(spec_add1_b)]
    fn add1_b(self) -> (r: Self)
        requires self@.a < u64::MAX - 1
        ensures r@ == self@.add1_b()
    {
        Self { a: self.a + 1 }
    }

    spec fn spec_add1_b(self) -> Self;
    broadcast proof fn view_add1_b_matches_spec_add1_b(self)
        ensures #![auto] self.spec_add1_b()@ == self@.add1_b()
    {
        admit();
    }
}

impl Abstract {
    spec fn add1_b(self) -> Self {
        Abstract {
            a: self.a + 1,
        }
    }
}

mod m1 {

    use super::*;
    broadcast use Concrete::view_add1_b_matches_spec_add1_b;

    fn exec_do(c0: Concrete) -> (c1: Concrete)
        requires c0@.a == 10,
        ensures c1@ == spec_do(c0@),
    {
        c0.add1_b()
    }

    spec fn spec_do(c0: Abstract) -> (c1: Abstract)
    {
        if c0.a == 10 {
            c0.add1_b()
        } else {
            arbitrary()
        }
    }
}

// Step 3 (inv_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Verified: -1, Errors: 999, Verus Errors: 2