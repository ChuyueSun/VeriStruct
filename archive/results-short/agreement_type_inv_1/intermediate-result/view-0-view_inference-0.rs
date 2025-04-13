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
        // TODO: add specification
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        // TODO: add specification
        match self, other {
            (AgreementResourceValue::Empty, x) => x,
            (x, AgreementResourceValue::Empty) => x,
            (AgreementResourceValue::Chosen { c: c1 }, AgreementResourceValue::Chosen { c: c2 }) =>
                if c1 == c2 {
                    AgreementResourceValue::Chosen { c: c1 }
                } else {
                    AgreementResourceValue::Invalid
                },
            _ => AgreementResourceValue::Invalid,
        }
    }

    open spec fn unit() -> Self {
        // TODO: add specification
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // TODO: add specification/proof
    }

    proof fn commutative(a: Self, b: Self) {
        // TODO: add specification/proof
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // TODO: add specification/proof
    }

    proof fn op_unit(a: Self) {
        // TODO: add specification/proof
    }

    proof fn unit_valid() {
        // TODO: add specification/proof
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        // TODO: add type invariant
        self.r@@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        // TODO: add specification
        self.r.id()
    }

    pub closed spec fn view(self) -> T
    {
        // TODO: add view
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            // If there's an 'Empty' or 'Invalid', we assume it won't occur under valid usage
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
        use_type_invariant(&*self);
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
    // TODO: add requires and ensures
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
// VerusErrorType.Other: expected curly braces
// {"$message_type":"diagnostic","message":"expected curly braces","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp7l4weelv","byte_start":803,"byte_end":804,"line_start":35,"line_end":35,"column_start":19,"column_end":20,"is_primary":true,"text":[{"text":"        match self, other {","highlight_start":19,"highlight_end":20}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected curly braces\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp7l4weelv:35:19\n   |\n35 |         match self, other {\n   |                   ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 