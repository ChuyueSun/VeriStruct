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
    // TODO:
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.Other: unclosed delimiterVerusErrorType.Other: this file contains an unclosed delimiter
// {"$message_type":"diagnostic","message":"this file contains an unclosed delimiter","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpot3_w_zr","byte_start":155,"byte_end":156,"line_start":9,"line_end":9,"column_start":8,"column_end":9,"is_primary":false,"text":[{"text":"verus! {","highlight_start":8,"highlight_end":9}],"label":"unclosed delimiter","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpot3_w_zr","byte_start":1324,"byte_end":1325,"line_start":66,"line_end":66,"column_start":30,"column_end":31,"is_primary":false,"text":[{"text":"impl<T> AgreementResource<T> {","highlight_start":30,"highlight_end":31}],"label":"unclosed delimiter","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpot3_w_zr","byte_start":2372,"byte_end":2372,"line_start":105,"line_end":105,"column_start":13,"column_end":13,"is_primary":true,"text":[{"text":"    // TODO:","highlight_start":13,"highlight_end":13}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: this file contains an unclosed delimiter\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpot3_w_zr:105:13\n    |\n9   | verus! {\n    |        - unclosed delimiter\n...\n66  | impl<T> AgreementResource<T> {\n    |                              - unclosed delimiter\n...\n105 |     // TODO:\n    |             ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 