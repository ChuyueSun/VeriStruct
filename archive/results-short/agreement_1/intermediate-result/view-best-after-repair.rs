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
        self.r.model().valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
    }

    pub closed spec fn view(self) -> (bool, T)
        recommends
            self.inv(),
    {
        match self.r.model() {
            AgreementResourceValue::Chosen { c } => (true, c),
            _ => (false, arbitrary()),
        }
    }

    pub proof fn alloc(c: T) -> (tracked res: AgreementResource<T>)
        requires
            true,
        ensures
            res.inv(),
            res@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        requires
            self.inv(),
        ensures
            self.inv(),
            result.inv(),
            result@ == self@,
    {
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
        ensures
            self.inv(),
            other.inv(),
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
// VerusErrorType.Other: no method named `model` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `model` found for struct `vstd::pcm::Resource` in the current scope
// {"$message_type":"diagnostic","message":"no method named `model` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym","byte_start":1867,"byte_end":1872,"line_start":76,"line_end":76,"column_start":16,"column_end":21,"is_primary":true,"text":[{"text":"        self.r.model().valid()","highlight_start":16,"highlight_end":21}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `model` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym:76:16\n   |\n76 |         self.r.model().valid()\n   |                ^^^^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym","byte_start":1946,"byte_end":1948,"line_start":80,"line_end":80,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id()","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym:80:16\n   |\n80 |         self.r.id()\n   |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `model` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym","byte_start":2075,"byte_end":2080,"line_start":87,"line_end":87,"column_start":22,"column_end":27,"is_primary":true,"text":[{"text":"        match self.r.model() {","highlight_start":22,"highlight_end":27}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `model` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpdkdf54ym:87:22\n   |\n87 |         match self.r.model() {\n   |                      ^^^^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 