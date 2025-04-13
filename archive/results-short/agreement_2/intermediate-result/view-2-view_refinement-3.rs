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
//!
//! An agreement resource constitutes knowledge of a constant value.
//! To create an instance of a constant value of type `T`, use
//! `AgreementResource::<T>::alloc()` as in the following example:
//!
//! ```
//! let tracked r1 = AgreementResource::<int>::alloc(72);
//! assert(r1@ == 72);
//! ```
//!
//! Knowledge of a constant value can be duplicated with `duplicate`,
//! which creates another agreement resource with the same constant
//! value and the same ID. Here's an example:
//!
//! ```
//! let tracked r2 = r1.duplicate();
//! assert(r2.id() == r1.id());
//! assert(r2@ == r1@);
//! ```
//!
//! Any two agreement resources with the same `id()` are guaranteed to
//! have equal values. You can establish this by calling
//! `lemma_agreement`, as in the following example:
//!
//! ```
//! assert(r2.id() == r1.id());
//! proof { r1.lemma_agreement(&mut r2); }
//! assert(r2@ == r1@);
//! ```

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
        /* TODO: part of view */
    }
}

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Invalid => false,
            _ => true,
        }
        /* TODO: part of view */
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Chosen { c: c2 } => if c == c2 {
                    AgreementResourceValue::Chosen { c }
                } else {
                    AgreementResourceValue::Invalid
                },
                AgreementResourceValue::Empty => AgreementResourceValue::Chosen { c },
            },
            AgreementResourceValue::Empty => other,
        }
        /* TODO: part of view */
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
        /* TODO: part of view */
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        /* TODO: part of view */
    }

    proof fn commutative(a: Self, b: Self) {
        /* TODO: part of view */
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        /* TODO: part of view */
    }

    proof fn op_unit(a: Self) {
        /* TODO: part of view */
    }

    proof fn unit_valid() {
        /* TODO: part of view */
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
        /* TODO: part of view */
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
        /* TODO: part of view */
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
        /* TODO: part of view */
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
    // TODO: add requires and ensures
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result:
        AgreementResource<T>)
    // TODO: add requires and ensures
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
    // TODO: add requires and ensures
    {
        self.r.validate_2(&other.r);
    }
}

impl<T> View for AgreementResource<T> {
    type V = (bool, Loc, T);

    closed spec fn view(&self) -> Self::V {
        (self.inv(), self.id(), self.view())
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    assert(r1@ == 72);
    let tracked r2 = r1.duplicate();
    assert(r2@ == r1@);
    proof {
        r1.lemma_agreement(&mut r2);
    }
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: expected square brackets
// {"$message_type":"diagnostic","message":"expected square brackets","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpw1wv_kjq","byte_start":158,"byte_end":227,"line_start":11,"line_end":11,"column_start":1,"column_end":70,"is_primary":true,"text":[{"text":"//! This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":70}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected square brackets\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpw1wv_kjq:11:1\n   |\n11 | //! This file implements agreement on a constant value using a custom\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 