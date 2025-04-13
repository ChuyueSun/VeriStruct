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

impl<T: PartialEq> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen{ .. } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen{ c } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen{ c: d } => if c == d {
                    AgreementResourceValue::Chosen{ c }
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

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (t: T) {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => unreachable(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        ensures
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        ensures
            result@ == old(self)@,
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
            old(self).id() == old(other).id(),
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
// VerusErrorType.Other: expected function, found macro `unreachable`
// {"$message_type":"diagnostic","message":"expected function, found macro `unreachable`","code":{"code":"E0423","explanation":"An identifier was used like a function name or a value was expected and the\nidentifier exists but it belongs to a different namespace.\n\nErroneous code example:\n\n```compile_fail,E0423\nstruct Foo { a: bool };\n\nlet f = Foo();\n// error: expected function, tuple struct or tuple variant, found `Foo`\n// `Foo` is a struct name, but this expression uses it like a function name\n```\n\nPlease verify you didn't misspell the name of what you actually wanted to use\nhere. Example:\n\n```\nfn Foo() -> u32 { 0 }\n\nlet f = Foo(); // ok!\n```\n\nIt is common to forget the trailing `!` on macro invocations, which would also\nyield this error:\n\n```compile_fail,E0423\nprintln(\"\");\n// error: expected function, tuple struct or tuple variant,\n// found macro `println`\n// did you mean `println!(...)`? (notice the trailing `!`)\n```\n\nAnother case where this error is emitted is when a value is expected, but\nsomething else is found:\n\n```compile_fail,E0423\npub mod a {\n    pub const I: i32 = 1;\n}\n\nfn h1() -> i32 {\n    a.I\n    //~^ ERROR expected value, found module `a`\n    // did you mean `a::I`?\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpit6wvy3p","byte_start":2088,"byte_end":2099,"line_start":85,"line_end":85,"column_start":18,"column_end":29,"is_primary":true,"text":[{"text":"            _ => unreachable(),","highlight_start":18,"highlight_end":29}],"label":"not a function","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"use `!` to invoke the macro","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpit6wvy3p","byte_start":2099,"byte_end":2099,"line_start":85,"line_end":85,"column_start":29,"column_end":29,"is_primary":true,"text":[{"text":"            _ => unreachable(),","highlight_start":29,"highlight_end":29}],"label":null,"suggested_replacement":"!","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null},{"message":"consider importing this function instead","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpit6wvy3p","byte_start":26,"byte_end":26,"line_start":2,"line_end":2,"column_start":1,"column_end":1,"is_primary":true,"text":[{"text":"use builtin::*;","highlight_start":1,"highlight_end":1}],"label":null,"suggested_replacement":"use std::intrinsics::unreachable;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0423]: expected function, found macro `unreachable`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpit6wvy3p:85:18\n   |\n85 |             _ => unreachable(),\n   |                  ^^^^^^^^^^^ not a function\n   |\nhelp: use `!` to invoke the macro\n   |\n85 |             _ => unreachable!(),\n   |                             +\nhelp: consider importing this function instead\n   |\n2  + use std::intrinsics::unreachable;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0423`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0423`.\n"}
// 
// 