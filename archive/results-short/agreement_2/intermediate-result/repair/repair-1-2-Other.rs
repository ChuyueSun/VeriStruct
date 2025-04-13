#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

#[verifier::loop_isolation(false)]

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

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Invalid => false,
            _ => true,
        }
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
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
    }

    proof fn commutative(a: Self, b: Self) {
    }

    proof fn associative(a: Self, b: Self, c: Self) {
    }

    proof fn op_unit(a: Self) {
    }

    proof fn unit_valid() {
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            true,
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
            result.inv(),
            self.id() == old(self).id(),
            result.id() == old(self).id(),
            self@ == old(self)@,
            result@ == old(self)@,
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        requires
            old(self).inv(),
            old(other).inv(),
        ensures
            self.inv(),
            other.inv(),
            self@ == old(self)@,
            other@ == old(other)@,
            if self.id() == other.id() { self@ == other@ } else { true },
    {
        self.r.validate_2(&other.r);
    }
}

impl<T> View for AgreementResource<T> {
    type V = (Loc, Option<T>);

    closed spec fn view(&self) -> Self::V {
        let val = match self.r@ {
            AgreementResourceValue::Chosen { c } => Some(c),
            _ => None,
        };
        (self.id(), val)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 8