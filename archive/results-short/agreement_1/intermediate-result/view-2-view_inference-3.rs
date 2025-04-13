
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
        // BEGIN: part of view
        AgreementResourceValue::Chosen { c }
        // END: part of view
    }
}

impl<T> PCM for AgreementResourceValue<T>
    where
        T: Eq, // Needed for comparing values in 'op'
{
    open spec fn valid(self) -> bool {
        // BEGIN: part of view
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
        // END: part of view
    }

    open spec fn op(self, other: Self) -> Self {
        // BEGIN: part of view
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Chosen { c: c1 } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Chosen { c: c2 } => {
                    if c1 == c2 {
                        AgreementResourceValue::Chosen { c: c1 }
                    } else {
                        AgreementResourceValue::Invalid
                    }
                }
            }
        }
        // END: part of view
    }

    open spec fn unit() -> Self {
        // BEGIN: part of view
        AgreementResourceValue::Empty
        // END: part of view
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        // No additional proof obligations needed for this PCM.
    }

    proof fn commutative(a: Self, b: Self) {
        // PCM operation is commutative by definition of 'op'.
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // PCM operation is associative by definition of 'op'.
    }

    proof fn op_unit(a: Self) {
        // 'Empty' is indeed the unit.
    }

    proof fn unit_valid() {
        // 'Empty' is valid.
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T>
    where
        T: Eq, // Needed for correctness of 'op'
{
    pub closed spec fn inv(self) -> bool {
        // BEGIN: part of view
        self.r@@.valid()
        && match self.r@@ {
            AgreementResourceValue::Chosen { c: _ } => true,
            _ => false,
        }
        // END: part of view
    }

    pub closed spec fn id(self) -> Loc {
        // BEGIN: part of view
        self.r.id()
        // END: part of view
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        // BEGIN: part of view
        match self.r@@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(), // unreachable if inv holds
        }
        // END: part of view
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        requires
            // no special requirements
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
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm","byte_start":2537,"byte_end":2538,"line_start":97,"line_end":97,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@@.valid()","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm:97:15\n   |\n97 |         self.r@@.valid()\n   |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm","byte_start":2571,"byte_end":2572,"line_start":98,"line_end":98,"column_start":24,"column_end":25,"is_primary":true,"text":[{"text":"        && match self.r@@ {","highlight_start":24,"highlight_end":25}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm:98:24\n   |\n98 |         && match self.r@@ {\n   |                        ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm","byte_start":2794,"byte_end":2796,"line_start":107,"line_end":107,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id()","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm:107:16\n    |\n107 |         self.r.id()\n    |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm","byte_start":2974,"byte_end":2975,"line_start":116,"line_end":116,"column_start":21,"column_end":22,"is_primary":true,"text":[{"text":"        match self.r@@ {","highlight_start":21,"highlight_end":22}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnr8twqfm:116:21\n    |\n116 |         match self.r@@ {\n    |                     ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 