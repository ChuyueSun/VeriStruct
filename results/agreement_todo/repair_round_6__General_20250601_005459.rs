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
        AgreementResourceValue::Chosen { c }
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
            AgreementResourceValue::Chosen { c: c1 } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: c2 } =>
                    if c1 == c2 {
                        AgreementResourceValue::Chosen { c: c1 }
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
        // No additional proof obligations needed for closed_under_incl beyond valid => a.valid()
    }

    proof fn commutative(a: Self, b: Self) {
        // The operation is commutative by inspection of the 'op' function design.
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // The operation is associative by standard agreement resource definition.
    }

    proof fn op_unit(a: Self) {
        // unit() is Empty, so a.op(Empty) == a and Empty.op(a) == a.
    }

    proof fn unit_valid() {
        // Empty is valid.
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        self.r.value().valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.loc()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        match self.r.value() {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(), // unreachable if self.inv() holds
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            true,
        ensures
            result.inv(),
            result@ == c,
            result.id() == result.r.loc(),
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result:
        AgreementResource<T>)
        requires
            old(self).inv(),
        ensures
            result.inv(),
            result.id() == old(self).id(),
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
            other.inv(),
            old(self).id() == other.id(),
        ensures
            self.inv(),
            other.inv(),
            self@ == other@,
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

// Repair Round 6 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1