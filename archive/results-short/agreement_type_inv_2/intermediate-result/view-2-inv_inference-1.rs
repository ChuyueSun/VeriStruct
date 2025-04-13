#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

//! This file implements agreement on a constant value using a custom
//! resource algebra.

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::<T>::Chosen { c }
    }
}

impl<T: PartialEq> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty
            | AgreementResourceValue::Chosen { .. } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (AgreementResourceValue::Empty, x) => x,
            (x, AgreementResourceValue::Empty) => x,
            (AgreementResourceValue::Chosen { c }, AgreementResourceValue::Chosen { c: d }) => {
                if c == d {
                    AgreementResourceValue::Chosen { c }
                } else {
                    AgreementResourceValue::Invalid
                }
            }
            _ => AgreementResourceValue::Invalid,
        }
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // Trivial, no additional partial order properties are required here.
    }

    proof fn commutative(a: Self, b: Self) {
        match (a, b) {
            (AgreementResourceValue::Empty, AgreementResourceValue::Empty) => { }
            (AgreementResourceValue::Empty, AgreementResourceValue::Chosen { .. }) => { }
            (AgreementResourceValue::Empty, AgreementResourceValue::Invalid) => { }
            (AgreementResourceValue::Chosen { .. }, AgreementResourceValue::Empty) => { }
            (AgreementResourceValue::Chosen { c }, AgreementResourceValue::Chosen { c: d }) => { }
            (AgreementResourceValue::Chosen { .. }, AgreementResourceValue::Invalid) => { }
            (AgreementResourceValue::Invalid, AgreementResourceValue::Empty) => { }
            (AgreementResourceValue::Invalid, AgreementResourceValue::Chosen { .. }) => { }
            (AgreementResourceValue::Invalid, AgreementResourceValue::Invalid) => { }
        };
        // By inspection of 'op', a.op(b) == b.op(a)
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // By inspection of 'op', the combination is associative in all cases.
        match (a, b, c) {
            (AgreementResourceValue::Empty, _, _)
            | (_, AgreementResourceValue::Empty, _)
            | (_, _, AgreementResourceValue::Empty) => { }
            (AgreementResourceValue::Chosen { c: c1 },
             AgreementResourceValue::Chosen { c: c2 },
             AgreementResourceValue::Chosen { c: c3 }) => { }
            _ => { }
        };
    }

    proof fn op_unit(a: Self) {
        // a op unit = a, unit op a = a
        // Follows from definition of 'Empty' as the unit.
    }

    proof fn unit_valid() {
        // Empty is valid by definition of 'valid'.
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: PartialEq> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (Loc, T) {
        reveal(self.inv());
        let c = match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        };
        (self.id(), c)
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
    {
        use_type_invariant(&*self);
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
    {
        use_type_invariant(&*self);
        use_type_invariant(&other);
        self.r.validate_2(&other.r);
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    assert(r1@ == 72);
    let tracked r2 = r1.duplicate();
    assert(r2.id() == r1.id());
    proof { r1.lemma_agreement(&mut r2); }
    assert(r2@ == r1@);
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: expected square brackets
// {"$message_type":"diagnostic","message":"expected square brackets","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv7syep72","byte_start":158,"byte_end":227,"line_start":11,"line_end":11,"column_start":1,"column_end":70,"is_primary":true,"text":[{"text":"//! This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":70}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected square brackets\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpv7syep72:11:1\n   |\n11 | //! This file implements agreement on a constant value using a custom\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 