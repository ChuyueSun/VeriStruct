#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

#[verifier::loop_isolation(false)]

// This file implements agreement on a constant value using a custom
// resource algebra.
//
// An agreement resource constitutes knowledge of a constant value.
// To create an instance of a constant value of type `T`, use
// `AgreementResource::<T>::alloc()` as in the following example:
//
// 
// let tracked r1 = AgreementResource::<int>::alloc(72);
// assert(r1@ == 72);
// 
//
// Knowledge of a constant value can be duplicated with `duplicate`,
// which creates another agreement resource with the same constant
// value and the same ID. Here's an example:
//
// 
// let tracked r2 = r1.duplicate();
// assert(r2.id() == r1.id());
// assert(r2@ == r1@);
// 
//
// Any two agreement resources with the same `id()` are guaranteed to
// have equal values. You can establish this by calling
// `lemma_agreement`, as in the following example:
//
// 
// assert(r2.id() == r1.id());
// proof { r1.lemma_agreement(&mut r2); }
// assert(r2@ == r1@);
// 

pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T: PartialEq> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
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
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c: c1 } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: c2 } => {
                    if c1 == c2 {
                        self
                    } else {
                        AgreementResourceValue::Invalid
                    }
                }
            },
        }
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(_a: Self, _b: Self) {
    }

    proof fn commutative(_a: Self, _b: Self) {
    }

    proof fn associative(_a: Self, _b: Self, _c: Self) {
    }

    proof fn op_unit(_a: Self) {
    }

    proof fn unit_valid() {
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: PartialEq> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        &&& self.r@.valid()
        &&& match self.r@ {
            AgreementResourceValue::Chosen { .. } => true,
            _ => false,
        }
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (Loc, T)
        recommends
            self.inv(),
    {
        (
            self.id(),
            match self.r@ {
                AgreementResourceValue::Chosen { c } => c,
                _ => arbitrary(),
            }
        )
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
        ensures
            result.inv(),
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            self@ == old(self)@,
            self.id() == old(self).id(),
            result.inv(),
            result.id() == self.id(),
            result@ == self@,
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    ) -> ()
        requires
            old(self).inv(),
            other.inv(),
        ensures
            self.inv(),
            other.inv(),
            self@ == old(self)@,
            if self.id() == other.id() {
                self@ == other@
            },
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

// //! This file implements agreement on a constant value using a custom
//   None: //! This file implements agreement on a constant value using a custom

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5