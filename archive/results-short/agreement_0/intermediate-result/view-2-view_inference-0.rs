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
        /* TODO: implement specification. */
        AgreementResourceValue::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T>
    where T: Equality
{
    open spec fn valid(self) -> bool {
        /* TODO: implement specification. */
        // We allow Empty or Chosen to be valid; Invalid is not valid.
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        /* TODO: implement specification. */
        // Merge logic:
        // - Empty op x = x
        // - x op Empty = x
        // - Chosen c op Chosen d = if c == d then Chosen c else Invalid
        // - Invalid op _ = Invalid
        // - _ op Invalid = Invalid
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: d } =>
                    if c == d {
                        AgreementResourceValue::Chosen { c }
                    } else {
                        AgreementResourceValue::Invalid
                    },
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            },
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
        }
    }

    open spec fn unit() -> Self {
        /* TODO: implement specification. */
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
        /* TODO: implement specification. */
        self.r@.valid()
        && (match self.r@ {
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Invalid => false,
        })
    }

    pub closed spec fn id(self) -> Loc {
        /* TODO: implement specification. */
        // The Resource type generally provides an ID we can use to distinguish.
        self.r.id()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        /* TODO: implement specification. */
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            // Because of the recommends self.inv(), in practice,
            // callers shouldn't see these cases. We'll just return
            // an arbitrary value in the spec.
            _ => arbitrary()
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
// {"$message_type":"diagnostic","message":"expected square brackets","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpue2f1fr1","byte_start":158,"byte_end":227,"line_start":11,"line_end":11,"column_start":1,"column_end":70,"is_primary":true,"text":[{"text":"//! This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":70}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected square brackets\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpue2f1fr1:11:1\n   |\n11 | //! This file implements agreement on a constant value using a custom\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 