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
        // TODO: part of view
        AgreementResourceValue::Chosen { c }
    }
}

impl<T: PartialEq> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        // TODO: part of view
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        // TODO: part of view
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Chosen { c: d } => if c == d {
                    AgreementResourceValue::Chosen { c }
                } else {
                    AgreementResourceValue::Invalid
                },
                AgreementResourceValue::Empty => AgreementResourceValue::Chosen { c },
            },
            AgreementResourceValue::Empty => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Chosen { c: d } => AgreementResourceValue::Chosen { c: d },
                AgreementResourceValue::Empty => AgreementResourceValue::Empty,
            }
        }
    }

    open spec fn unit() -> Self {
        // TODO: part of view
        AgreementResourceValue::Empty
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // TODO: part of view
    }

    proof fn commutative(a: Self, b: Self) {
        // TODO: part of view
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // TODO: part of view
    }

    proof fn op_unit(a: Self) {
        // TODO: part of view
    }

    proof fn unit_valid() {
        // TODO: part of view
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T: PartialEq> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        // TODO: part of view
        self.r@.valid()
        && match self.r@ {
            AgreementResourceValue::Chosen { c: _ } => true,
            _ => false,
        }
    }

    pub closed spec fn id(self) -> Loc {
        // TODO: part of view
        self.r.id()
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        // TODO: part of view
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        // TODO: part of view
        requires
            true,
        ensures
            result.inv(),
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        // TODO: part of view
        requires
            self.inv(),
        ensures
            result.inv(),
            result@ == self@,
            result.id() == self.id(),
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        // TODO: part of view
        requires
            self.inv(),
            other.inv(),
        ensures
            if self.id() == other.id() {
                self@ == other@
            } else {
                true
            },
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scope
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o","byte_start":2458,"byte_end":2459,"line_start":88,"line_end":88,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@.valid()","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o:88:15\n   |\n88 |         self.r@.valid()\n   |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o","byte_start":2491,"byte_end":2492,"line_start":89,"line_end":89,"column_start":24,"column_end":25,"is_primary":true,"text":[{"text":"        && match self.r@ {","highlight_start":24,"highlight_end":25}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o:89:24\n   |\n89 |         && match self.r@ {\n   |                        ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o","byte_start":2683,"byte_end":2685,"line_start":97,"line_end":97,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id()","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o:97:16\n   |\n97 |         self.r.id()\n   |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o","byte_start":2833,"byte_end":2834,"line_start":105,"line_end":105,"column_start":21,"column_end":22,"is_primary":true,"text":[{"text":"        match self.r@ {","highlight_start":21,"highlight_end":22}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpa4p0j5_o:105:21\n    |\n105 |         match self.r@ {\n    |                     ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 