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

    proof fn closed_under_incl(_a: Self, _b: Self) {}
    proof fn commutative(_a: Self, _b: Self) {}
    proof fn associative(_a: Self, _b: Self, _c: Self) {}
    proof fn op_unit(_a: Self) {}
    proof fn unit_valid() {}
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    /// Type invariant: The underlying resource value must be valid
    #[verifier::type_invariant]
    pub closed spec fn inv(&self) -> bool {
        self.r.value().valid()
    }

    /// Return the “location” or identifier for this resource
    pub closed spec fn id(self) -> Loc {
        self.r.loc()
    }

    /// New version of view: returns a plain `T` so you can do `assert(r1@ == 72)`.
    /// If you require that the resource is actually chosen, then for the other variants
    /// we can pick an arbitrary value (or do something else if you want stricter checks).
    pub closed spec fn view(self) -> T {
        match self.r.value() {
            AgreementResourceValue::Chosen { c } => c,
            AgreementResourceValue::Empty => arbitrary(),
            AgreementResourceValue::Invalid => arbitrary(),
        }
    }

    /// Provide a constructor that ensures the resource is indeed `Chosen(c)`.
    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            AgreementResourceValue::<T>::new(c).valid(),
        ensures
            // Now you can simply check `assert(result@ == c)` at callsites
            result@ == c,
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
            result@ == self@,
    {
        use_type_invariant(&*self);
        let tracked r_copy = duplicate(&self.r);
        AgreementResource::<T> { r: r_copy }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        requires
            old(self).id() == other.id(),
        ensures
            self@ == other@,
    {
        use_type_invariant(&*self);
        use_type_invariant(&other);
        self.r.validate_2(&other.r);
    }
}

pub fn main() {
    let tracked r1 = AgreementResource::<int>::alloc(72);
    // Now r1@ is of type int, so this is valid:
    assert(r1@ == 72);

    let tracked r2 = r1.duplicate();
    assert(r2.id() == r1.id());

    proof {
        r1.lemma_agreement(&mut r2);
    }
    assert(r2@ == r1@);
}

}
