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
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { .. } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: c_other } => if c == c_other {
                    self
                } else {
                    AgreementResourceValue::Invalid
                },
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            },
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
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

impl<T: PartialEq> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (bool, T)
        recommends
            self.inv(),
    {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => (true, c),
            _ => (false, arbitrary()),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            true,
        ensures
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        requires
            let old_self = old(self),
        ensures
            self@ == old_self@,
            result@ == old_self@,
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    ) -> (ret: ())
        requires
            let old_self = old(self),
            let old_other = old(other),
        ensures
            self@ == old_self@,
            other@ == old_other@,
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: expected expression, found `let` statementVerusErrorType.Other: expected expression, found `let` statementVerusErrorType.Other: expected expression, found `let` statement
// {"$message_type":"diagnostic","message":"expected expression, found `let` statement","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh","byte_start":2631,"byte_end":2634,"line_start":104,"line_end":104,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"            let old_self = old(self),","highlight_start":13,"highlight_end":16}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"only supported directly in conditions of `if` and `while` expressions","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error: expected expression, found `let` statement\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh:104:13\n    |\n104 |             let old_self = old(self),\n    |             ^^^\n    |\n    = note: only supported directly in conditions of `if` and `while` expressions\n\n"}
// {"$message_type":"diagnostic","message":"expected expression, found `let` statement","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh","byte_start":3010,"byte_end":3013,"line_start":118,"line_end":118,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"            let old_self = old(self),","highlight_start":13,"highlight_end":16}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"only supported directly in conditions of `if` and `while` expressions","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error: expected expression, found `let` statement\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh:118:13\n    |\n118 |             let old_self = old(self),\n    |             ^^^\n    |\n    = note: only supported directly in conditions of `if` and `while` expressions\n\n"}
// {"$message_type":"diagnostic","message":"expected expression, found `let` statement","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh","byte_start":3048,"byte_end":3051,"line_start":119,"line_end":119,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"            let old_other = old(other),","highlight_start":13,"highlight_end":16}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"only supported directly in conditions of `if` and `while` expressions","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error: expected expression, found `let` statement\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp97iyv6dh:119:13\n    |\n119 |             let old_other = old(other),\n    |             ^^^\n    |\n    = note: only supported directly in conditions of `if` and `while` expressions\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// 
// 