
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
        /* TODO: part of view */
        // e.g., AgreementResourceValue::Chosen { c }
        unimplemented!()
    }
}

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        /* TODO: part of view */
        unimplemented!()
    }

    open spec fn op(self, other: Self) -> Self {
        /* TODO: part of view */
        unimplemented!()
    }

    open spec fn unit() -> Self {
        /* TODO: part of view */
        unimplemented!()
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
    pub closed spec fn inv(self) -> bool {
        /* TODO: part of view */
        true
    }

    pub closed spec fn id(self) -> Loc {
        /* TODO: part of view */
        arbitrary()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        /* TODO: part of view */
        unimplemented!()
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
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
    // TODO: add requires and ensures
    {
        self.r.validate_2(&other.r);
    }
}

impl<T> View for AgreementResource<T> {
    type V = T;

    closed spec fn view(&self) -> Self::V {
        /* TODO: part of view */
        AgreementResource::<T>::view(*self)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: panic is not supported (if you used Rust's `assert!` macro, you may have meant to use Verus's `assert` function)
// {"$message_type":"diagnostic","message":"panic is not supported (if you used Rust's `assert!` macro, you may have meant to use Verus's `assert` function)","code":null,"level":"error","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs","byte_start":29020,"byte_end":29063,"line_start":837,"line_end":837,"column_start":9,"column_end":52,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":{"span":{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwlkw9eor","byte_start":416,"byte_end":432,"line_start":22,"line_end":22,"column_start":9,"column_end":25,"is_primary":false,"text":[{"text":"        unimplemented!()","highlight_start":9,"highlight_end":25}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},"macro_decl_name":"unimplemented!","def_site_span":{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/macros/mod.rs","byte_start":28971,"byte_end":28997,"line_start":835,"line_end":835,"column_start":1,"column_end":27,"is_primary":false,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}}],"children":[],"rendered":"error: panic is not supported (if you used Rust's `assert!` macro, you may have meant to use Verus's `assert` function)\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpwlkw9eor:22:9\n   |\n22 |         unimplemented!()\n   |         ^^^^^^^^^^^^^^^^\n   |\n   = note: this error originates in the macro `unimplemented` (in Nightly builds, run with -Z macro-backtrace for more info)\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
