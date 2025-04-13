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
//! `AgreementResource::<T>::alloc(c)`. Knowledge of a constant value
//! can be duplicated with `duplicate`, which creates another agreement
//! resource with the same constant value and the same ID. Any two
//! agreement resources with the same `id()` are guaranteed to
//! have equal values, which can be established with `lemma_agreement`.

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T>
    where T: PartialEq
{
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Invalid => false,
            _ => true,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c: c1 } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: c2 } =>
                    if c1 == c2 {
                        self
                    } else {
                        AgreementResourceValue::Invalid
                    },
            },
        }
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // If valid(a) && valid(b), then valid(a op b).
        match a {
            AgreementResourceValue::Invalid => { /* invalid(a) => no obligation */ }
            AgreementResourceValue::Empty => { /* a op b = b => valid if valid(b) */ }
            AgreementResourceValue::Chosen { c: c1 } => match b {
                AgreementResourceValue::Invalid => { /* invalid(b) => no obligation */ }
                AgreementResourceValue::Empty => { /* a op b = a => valid if valid(a) */ }
                AgreementResourceValue::Chosen { c: c2 } => {
                    /* a op b is either Chosen(c1) if c1 == c2, or Invalid otherwise */
                }
            },
        }
    }

    proof fn commutative(a: Self, b: Self) {
        // a op b = b op a
        match a {
            AgreementResourceValue::Invalid => { /* Invalid op anything = Invalid */ }
            AgreementResourceValue::Empty => { /* Empty op b = b, b op Empty = b */ }
            AgreementResourceValue::Chosen { c: c1 } => match b {
                AgreementResourceValue::Invalid => { /* a op b = Invalid, b op a = Invalid */ }
                AgreementResourceValue::Empty => { /* a op b = a, b op a = a */ }
                AgreementResourceValue::Chosen { c: c2 } => {
                    /* if c1 == c2 => a op b = a = b op a; else => Invalid for both orders */
                }
            },
        }
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // (a op b) op c = a op (b op c)
        match a {
            AgreementResourceValue::Invalid => { /* invalid => all sides invalid */ }
            AgreementResourceValue::Empty => { /* (Empty op b) op c = b op c, Empty op (b op c) = b op c */ }
            AgreementResourceValue::Chosen { c: c1 } => {
                match b {
                    AgreementResourceValue::Invalid => { /* invalid => all sides invalid */ }
                    AgreementResourceValue::Empty => { /* a op b = a => a op c versus a op (Empty op c) => a op c */ }
                    AgreementResourceValue::Chosen { c: c2 } => {
                        match c {
                            AgreementResourceValue::Invalid => { /* invalid => all sides invalid */ }
                            AgreementResourceValue::Empty => { /* b op c = b => a op b => handle above match */ }
                            AgreementResourceValue::Chosen { c: c3 } => {
                                // if c1 == c2 and c2 == c3 => c1 == c3 => all Chosen => associativity holds
                                // if any mismatch => Invalid => all invalid => trivially equal
                            }
                        }
                    }
                }
            }
        }
    }

    proof fn op_unit(a: Self) {
        // a op unit() = a and unit() op a = a
        match a {
            AgreementResourceValue::Invalid => { }
            AgreementResourceValue::Empty => { }
            AgreementResourceValue::Chosen { c: _ } => { }
        }
    }

    proof fn unit_valid() {
        // valid(unit())
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (Loc, bool, T) {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => (self.id(), true, c),
            _ => (self.id(), false, arbitrary()),
        }
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
// {"$message_type":"diagnostic","message":"expected square brackets","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn1akctdc","byte_start":158,"byte_end":227,"line_start":11,"line_end":11,"column_start":1,"column_end":70,"is_primary":true,"text":[{"text":"//! This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":70}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected square brackets\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpn1akctdc:11:1\n   |\n11 | //! This file implements agreement on a constant value using a custom\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 