
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
        /* TODO: part of view */
        AgreementResourceValue::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T>
    where
        T: PartialEq
{
    open spec fn valid(self) -> bool {
        /* TODO: part of view */
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { .. } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        /* TODO: part of view */
        match (self, other) {
            (AgreementResourceValue::Empty, x) => x,
            (x, AgreementResourceValue::Empty) => x,
            (AgreementResourceValue::Chosen { c }, AgreementResourceValue::Chosen { c: c2 }) =>
                if c == c2 {
                    AgreementResourceValue::Chosen { c }
                } else {
                    AgreementResourceValue::Invalid
                },
            _ => AgreementResourceValue::Invalid,
        }
    }

    open spec fn unit() -> Self {
        /* TODO: part of view */
        AgreementResourceValue::Empty
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
        /* TODO: part of view */
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        /* TODO: part of view */
        self.r.id()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        /* TODO: part of view */
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            // If we're in a valid resource, we expect `Empty` or `Chosen`. But `Empty` has no T.
            // We'll treat that as unreachable in contexts where `view` is used.
            // Another option is to return an arbitrary T.
            _ => arbitrary(),
        }
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
    type V = T;

    closed spec fn view(&self) -> Self::V {
        self.view()
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
// {"$message_type":"diagnostic","message":"expected square brackets","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpprme07co","byte_start":159,"byte_end":228,"line_start":12,"line_end":12,"column_start":1,"column_end":70,"is_primary":true,"text":[{"text":"//! This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":70}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected square brackets\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpprme07co:12:1\n   |\n12 | //! This file implements agreement on a constant value using a custom\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 