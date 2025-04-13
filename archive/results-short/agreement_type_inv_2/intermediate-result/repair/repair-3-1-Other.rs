#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

#[verifier(external_body)]
impl<T> Resource<T> {
    pub closed spec fn view(&self) -> T {
        arbitrary()
    }
}

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
            AgreementResourceValue::Chosen { .. } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (AgreementResourceValue::Invalid, _) => AgreementResourceValue::Invalid,
            (_, AgreementResourceValue::Invalid) => AgreementResourceValue::Invalid,
            (AgreementResourceValue::Empty, r) => r,
            (l, AgreementResourceValue::Empty) => l,
            (AgreementResourceValue::Chosen { c: c1 }, AgreementResourceValue::Chosen { c: c2 }) => {
                if c1 == c2 {
                    AgreementResourceValue::Chosen { c: c1 }
                } else {
                    AgreementResourceValue::Invalid
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
        self.r.view().valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn info(self) -> (Loc, bool, Option<T>) {
        let id = self.r.id();
        match self.r.view() {
            AgreementResourceValue::Empty => (id, false, None),
            AgreementResourceValue::Chosen { c } => (id, true, Some(c)),
            AgreementResourceValue::Invalid => arbitrary(),
        }
    }

    pub closed spec fn ghost_val(self) -> T {
        match self.r.view() {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            true,
        ensures
            result.ghost_val() == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        requires
            true,
        ensures
            result.id() == self.id(),
            result.ghost_val() == self.ghost_val(),
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
            true,
        ensures
            self.id() == other.id() ==> self.ghost_val() == other.ghost_val(),
    {
        use_type_invariant(&*self);
        use_type_invariant(&other);
        self.r.validate_2(&other.r);
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    assert(r1.ghost_val() == 72);
    let tracked r2 = r1.duplicate();
    assert(r2.id() == r1.id());
    proof { r1.lemma_agreement(&mut r2); }
    assert(r2.ghost_val() == r1.ghost_val());
}

}

//         match self.r@ {
//   method not found in `Resource<AgreementResourceValue<T>>`: @

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 17