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

impl<T> PCM for AgreementResourceValue<T>
    where T: Equality
{
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c } => true, /* TODO: part of view */
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (AgreementResourceValue::Invalid, _) | (_, AgreementResourceValue::Invalid) => AgreementResourceValue::Invalid,
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
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (u8, T) {
        match self.r@ {
            AgreementResourceValue::Empty => (0, arbitrary()),
            AgreementResourceValue::Chosen { c } => (1, c),
            AgreementResourceValue::Invalid => (2, arbitrary()),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        ensures
            result.view().0 == 1,
            result.view().1 == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        ensures
            result.view() == old(self).view(),
            result.id() == old(self).id(),
    {
        use_type_invariant(&*self);
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    ) -> ()
        ensures
            self.view() == old(self).view(),
            other.view() == old(other).view(),
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
// VerusErrorType.Other: cannot find trait `Equality` in this scope
// {"$message_type":"diagnostic","message":"cannot find trait `Equality` in this scope","code":{"code":"E0405","explanation":"The code refers to a trait that is not in scope.\n\nErroneous code example:\n\n```compile_fail,E0405\nstruct Foo;\n\nimpl SomeTrait for Foo {} // error: trait `SomeTrait` is not in scope\n```\n\nPlease verify that the name of the trait wasn't misspelled and ensure that it\nwas imported. Example:\n\n```\n# #[cfg(for_demonstration_only)]\n// solution 1:\nuse some_file::SomeTrait;\n\n// solution 2:\ntrait SomeTrait {\n    // some functions\n}\n\nstruct Foo;\n\nimpl SomeTrait for Foo { // ok!\n    // implements functions\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpp_tu4tb4","byte_start":429,"byte_end":437,"line_start":24,"line_end":24,"column_start":14,"column_end":22,"is_primary":true,"text":[{"text":"    where T: Equality","highlight_start":14,"highlight_end":22}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0405]: cannot find trait `Equality` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpp_tu4tb4:24:14\n   |\n24 |     where T: Equality\n   |              ^^^^^^^^ not found in this scope\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0405`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0405`.\n"}
//
//
