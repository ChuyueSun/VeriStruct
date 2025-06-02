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
        AgreementResourceValue::<T>::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c: c1 } => {
                match other {
                    AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                    AgreementResourceValue::Empty => AgreementResourceValue::Chosen { c: c1 },
                    AgreementResourceValue::Chosen { c: c2 } => {
                        if c1 == c2 {
                            AgreementResourceValue::Chosen { c: c1 }
                        } else {
                            AgreementResourceValue::Invalid
                        }
                    }
                }
            }
        }
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(_a: Self, _b: Self) { }

    proof fn commutative(_a: Self, _b: Self) { }

    proof fn associative(_a: Self, _b: Self, _c: Self) { }

    proof fn op_unit(_a: Self) { }

    proof fn unit_valid() { }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(&self) -> bool {
        self.r.value().valid()
        && match self.r.value() {
            AgreementResourceValue::Chosen { c: _ } => true,
            _ => false,
        }
    }

    pub closed spec fn id(&self) -> Loc {
        self.r.loc()
    }

    pub closed spec fn view(&self) -> (Loc, T) {
        match self.r.value() {
            AgreementResourceValue::Chosen { c } => (self.r.loc(), c),
            _ => (self.r.loc(), arbitrary()),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            true,
        ensures
            result@.1 == c,
            result.inv(),
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        ensures
            result@.1 == self@.1,
            result.id() == self.id(),
    {
        use_type_invariant(&*self);
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        requires
            old(self).id() == other.id(),
        ensures
            self@.1 == other@.1,
    {
        use_type_invariant(&*self);
        use_type_invariant(&other);
        self.r.validate_2(&other.r);
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    assert(r1@.1 == 72);
    let tracked r2 = r1.duplicate();
    assert(r2@.1 == r1@.1);
    proof { r1.lemma_agreement(&mut r2); }
}

} // verus!

// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1