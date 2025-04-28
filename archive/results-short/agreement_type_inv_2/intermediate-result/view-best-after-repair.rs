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
        AgreementResourceValue::<T>::Chosen { c }
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

    proof fn closed_under_incl(_a: Self, _b: Self) {
    }

    proof fn commutative(_a: Self, _b: Self) {
    }

    proof fn associative(_a: Self, _b: Self, _c: Self) {
    }

    proof fn op_unit(_a: Self) {
    }

    proof fn unit_valid() {
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(&self) -> bool {
        (*self.r.borrow()).valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (Loc, bool, Option<T>) {
        let id = self.r.id();
        match *self.r.borrow() {
            AgreementResourceValue::Empty => (id, false, None),
            AgreementResourceValue::Chosen { c } => (id, true, Some(c)),
            AgreementResourceValue::Invalid => arbitrary(),
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
            true,
        ensures
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
            true,
        ensures
            self.id() == other.id() ==> self@ == other@,
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

}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: no method named `borrow` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `borrow` found for struct `vstd::pcm::Resource` in the current scope
// {"$message_type":"diagnostic","message":"no method named `borrow` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/borrow.rs","byte_start":7103,"byte_end":7109,"line_start":178,"line_end":178,"column_start":8,"column_end":14,"is_primary":false,"text":[],"label":"the method is available for `vstd::pcm::Resource<AgreementResourceValue<T>>` here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":1912,"byte_end":1918,"line_start":76,"line_end":76,"column_start":18,"column_end":24,"is_primary":true,"text":[{"text":"        (*self.r.borrow()).valid()","highlight_start":18,"highlight_end":24}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"trait `Borrow` which provides `borrow` is implemented but not in scope; perhaps you want to import it","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":26,"byte_end":26,"line_start":2,"line_end":2,"column_start":1,"column_end":1,"is_primary":true,"text":[{"text":"use builtin::*;","highlight_start":1,"highlight_end":1}],"label":null,"suggested_replacement":"use std::borrow::Borrow;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null},{"message":"there is a method `borrow_mut` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":1912,"byte_end":1918,"line_start":76,"line_end":76,"column_start":18,"column_end":24,"is_primary":true,"text":[{"text":"        (*self.r.borrow()).valid()","highlight_start":18,"highlight_end":24}],"label":null,"suggested_replacement":"borrow_mut","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `borrow` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq:76:18\n   |\n76 |         (*self.r.borrow()).valid()\n   |                  ^^^^^^\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/borrow.rs:178:8\n   |\n   = note: the method is available for `vstd::pcm::Resource<AgreementResourceValue<T>>` here\n   |\n   = help: items from traits can only be used if the trait is in scope\nhelp: trait `Borrow` which provides `borrow` is implemented but not in scope; perhaps you want to import it\n   |\n2  + use std::borrow::Borrow;\n   |\nhelp: there is a method `borrow_mut` with a similar name\n   |\n76 |         (*self.r.borrow_mut()).valid()\n   |                  ~~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":1993,"byte_end":1995,"line_start":80,"line_end":80,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id()","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq:80:16\n   |\n80 |         self.r.id()\n   |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":2091,"byte_end":2093,"line_start":84,"line_end":84,"column_start":25,"column_end":27,"is_primary":true,"text":[{"text":"        let id = self.r.id();","highlight_start":25,"highlight_end":27}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq:84:25\n   |\n84 |         let id = self.r.id();\n   |                         ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `borrow` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/borrow.rs","byte_start":7103,"byte_end":7109,"line_start":178,"line_end":178,"column_start":8,"column_end":14,"is_primary":false,"text":[],"label":"the method is available for `vstd::pcm::Resource<AgreementResourceValue<T>>` here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":2119,"byte_end":2125,"line_start":85,"line_end":85,"column_start":23,"column_end":29,"is_primary":true,"text":[{"text":"        match *self.r.borrow() {","highlight_start":23,"highlight_end":29}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"trait `Borrow` which provides `borrow` is implemented but not in scope; perhaps you want to import it","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":26,"byte_end":26,"line_start":2,"line_end":2,"column_start":1,"column_end":1,"is_primary":true,"text":[{"text":"use builtin::*;","highlight_start":1,"highlight_end":1}],"label":null,"suggested_replacement":"use std::borrow::Borrow;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null},{"message":"there is a method `borrow_mut` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq","byte_start":2119,"byte_end":2125,"line_start":85,"line_end":85,"column_start":23,"column_end":29,"is_primary":true,"text":[{"text":"        match *self.r.borrow() {","highlight_start":23,"highlight_end":29}],"label":null,"suggested_replacement":"borrow_mut","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `borrow` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpgbq9ukfq:85:23\n   |\n85 |         match *self.r.borrow() {\n   |                       ^^^^^^\n  --> /rustc/f6e511eec7342f59a25f7c0534f1dbea00d01b14/library/core/src/borrow.rs:178:8\n   |\n   = note: the method is available for `vstd::pcm::Resource<AgreementResourceValue<T>>` here\n   |\n   = help: items from traits can only be used if the trait is in scope\nhelp: trait `Borrow` which provides `borrow` is implemented but not in scope; perhaps you want to import it\n   |\n2  + use std::borrow::Borrow;\n   |\nhelp: there is a method `borrow_mut` with a similar name\n   |\n85 |         match *self.r.borrow_mut() {\n   |                       ~~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
//
//
