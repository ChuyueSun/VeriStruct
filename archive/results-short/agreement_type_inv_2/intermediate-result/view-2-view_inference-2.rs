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

impl<T: PartialEq> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
    }
}

impl<T: PartialEq> PCM for AgreementResourceValue<T> {
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
                AgreementResourceValue::Chosen { c: c2 } => if c1 == c2 {
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
        // trivial
    }

    proof fn commutative(a: Self, b: Self) {
        assert(Self::op(a, b) == Self::op(b, a));
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        assert(Self::op(Self::op(a, b), c) == Self::op(a, Self::op(b, c)));
    }

    proof fn op_unit(a: Self) {
        assert(Self::op(a, Self::unit()) == a);
        assert(Self::op(Self::unit(), a) == a);
    }

    proof fn unit_valid() {
        assert(Self::unit().valid());
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: PartialEq> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid() && match self.r@ {
            AgreementResourceValue::Chosen { _ } => true,
            _ => false,
        }
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> T {
        reveal_self(self.inv());
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            // By the type invariant, other cases cannot occur
            _ => arbitrary(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            // We can always allocate
            true,
        ensures
            result.inv(),
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result:
        AgreementResource<T>)
        requires
            self.inv(),
        ensures
            result.inv(),
            result.id() == self.id(),
            result@ == self@,
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
            self.inv(),
            other.inv(),
            self.id() == other.id(),
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
    assert(r1@ == 72);
    let tracked r2 = r1.duplicate();
    assert(r2.id() == r1.id());
    proof { r1.lemma_agreement(&mut r2); }
    assert(r2@ == r1@);
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: expected identifier or integer
// {"$message_type":"diagnostic","message":"expected identifier or integer","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpq2kj9zly","byte_start":2159,"byte_end":2160,"line_start":81,"line_end":81,"column_start":46,"column_end":47,"is_primary":true,"text":[{"text":"            AgreementResourceValue::Chosen { _ } => true,","highlight_start":46,"highlight_end":47}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected identifier or integer\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpq2kj9zly:81:46\n   |\n81 |             AgreementResourceValue::Chosen { _ } => true,\n   |                                              ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 