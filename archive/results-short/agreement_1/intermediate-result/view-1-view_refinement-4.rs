#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        // -- minimal specification for “new”: create a Chosen variant
        AgreementResourceValue::Chosen { c }
    }
}

impl<T: PartialEq> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        // -- “Invalid” is never valid, “Empty” and “Chosen” are valid
        match self {
            AgreementResourceValue::Invalid => false,
            _ => true,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c: c1 } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: c2 } =>
                    if c1 == c2 {
                        self
                    } else {
                        AgreementResourceValue::Invalid
                    },
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            },
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
        }
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // no additional proof obligations needed for this example
    }

    proof fn commutative(a: Self, b: Self) {
        // no additional proof obligations needed for this example
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // no additional proof obligations needed for this example
    }

    proof fn op_unit(a: Self) {
        // no additional proof obligations needed for this example
    }

    proof fn unit_valid() {
        // no additional proof obligations needed for this example
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: PartialEq> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        // The resource is valid and specifically “Chosen” for a well-formed AgreementResource
        matches!(self.r@, AgreementResourceValue::Chosen { .. })
    }

    pub closed spec fn id(self) -> Loc {
        // Return the underlying resource’s location
        self.r.loc
    }

    pub closed spec fn view(self) -> (Loc, T)
        recommends
            self.inv(),
    {
        // Return a flattened tuple of the resource location and the chosen value
        (
            self.r.loc,
            match self.r@ {
                AgreementResourceValue::Chosen { c } => c,
                _ => unreachable!(),
            }
        )
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        // For verification, we’d require nothing in particular:
        // requires true,
        // ensures result.inv(),
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        // For verification, we’d require self.inv()
        // ensures result@ == self@
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        // requires self.inv() && other.inv(),
        // ensures other@ == self@
    {
        self.r.validate_2(&other.r);
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    assert(r1@.1 == 72);
    let tracked r2 = r1.duplicate();
    assert(r2@.1 == r1@.1);
    proof {
        r1.lemma_agreement(&mut r2);
    }
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: no rules expected the token `@`
// {"$message_type":"diagnostic","message":"no rules expected the token `@`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_3c09r82","byte_start":2364,"byte_end":2365,"line_start":82,"line_end":82,"column_start":24,"column_end":25,"is_primary":true,"text":[{"text":"        matches!(self.r@, AgreementResourceValue::Chosen { .. })","highlight_start":24,"highlight_end":25}],"label":"no rules expected this token in macro call","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"while trying to match `,`","code":null,"level":"note","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs","byte_start":17480,"byte_end":17481,"line_start":474,"line_end":474,"column_start":22,"column_end":23,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error: no rules expected the token `@`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp_3c09r82:82:24\n   |\n82 |         matches!(self.r@, AgreementResourceValue::Chosen { .. })\n   |                        ^ no rules expected this token in macro call\n   |\nnote: while trying to match `,`\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs:474:22\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
