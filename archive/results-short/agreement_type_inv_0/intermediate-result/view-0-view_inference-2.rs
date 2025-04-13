#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

/// This file implements agreement on a constant value using a custom
/// resource algebra.
///
/// An agreement resource constitutes knowledge of a constant value.
/// To create an instance of a constant value of type `T`, use
/// `AgreementResource::<T>::alloc()` as in the following example:
///
/// ```
/// let tracked r1 = AgreementResource::<int>::alloc(72);
/// assert(r1@ == 72);
/// ```
///
/// Knowledge of a constant value can be duplicated with `duplicate`,
/// which creates another agreement resource with the same constant
/// value and the same ID. Here's an example:
///
/// ```
/// let tracked r2 = r1.duplicate();
/// assert(r2.id() == r1.id());
/// assert(r2@ == r1@);
/// ```
///
/// Any two agreement resources with the same `id()` are guaranteed to
/// have equal values. You can establish this by calling
/// `lemma_agreement`, as in the following example:
///
/// ```
/// assert(r2.id() == r1.id());
/// proof { r1.lemma_agreement(&mut r2); }
/// assert(r2@ == r1@);
/// ```
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

impl<T> PCM for AgreementResourceValue<T>
    where T: PartialEq
{
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,        /* TODO: part of view */
            AgreementResourceValue::Chosen { .. } => true,/* TODO: part of view */
            AgreementResourceValue::Invalid => false,     /* TODO: part of view */
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
        AgreementResourceValue::Empty /* TODO: part of view */
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        /* No additional conditions required for this PCM */
    }

    proof fn commutative(a: Self, b: Self) {
        /* The definition of op is commutative */
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        /* The definition of op is associative */
    }

    proof fn op_unit(a: Self) {
        /* a ⋅ unit = a */
    }

    proof fn unit_valid() {
        /* unit is valid */
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    #[verifier::type_invariant]
    pub closed spec fn inv(self) -> bool {
        self.r@.valid() /* TODO: part of view */
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id() /* TODO: part of view */
    }

    pub closed spec fn view(self) -> T {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(), /* Should never happen if inv(self) holds */
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
    // TODO: add requires and ensures
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 6
// VerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfiedVerusErrorType.Other: can't compare `T` with `T`VerusErrorType.Other: the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":3216,"byte_end":3217,"line_start":113,"line_end":113,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@.valid() /* TODO: part of view */","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:113:15\n    |\n113 |         self.r@.valid() /* TODO: part of view */\n    |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":3314,"byte_end":3316,"line_start":117,"line_end":117,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id() /* TODO: part of view */","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:117:16\n    |\n117 |         self.r.id() /* TODO: part of view */\n    |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":3412,"byte_end":3413,"line_start":121,"line_end":121,"column_start":21,"column_end":22,"is_primary":true,"text":[{"text":"        match self.r@ {","highlight_start":21,"highlight_end":22}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:121:21\n    |\n121 |         match self.r@ {\n    |                     ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":3802,"byte_end":3807,"line_start":131,"line_end":131,"column_start":66,"column_end":71,"is_primary":true,"text":[{"text":"        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);","highlight_start":66,"highlight_end":71}],"label":"function or associated item cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1158,"byte_end":1192,"line_start":42,"line_end":42,"column_start":1,"column_end":35,"is_primary":false,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":1,"highlight_end":35}],"label":"doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"trait bound `T: std::cmp::PartialEq` was not satisfied","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1434,"byte_end":1443,"line_start":55,"line_end":55,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1387,"byte_end":1390,"line_start":54,"line_end":54,"column_start":9,"column_end":12,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1395,"byte_end":1420,"line_start":54,"line_end":54,"column_start":17,"column_end":42,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting the type parameter to satisfy the trait bound","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1192,"byte_end":1192,"line_start":42,"line_end":42,"column_start":35,"column_end":35,"is_primary":true,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":35,"highlight_end":35}],"label":null,"suggested_replacement":" where T: std::cmp::PartialEq","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:131:66\n    |\n42  | pub enum AgreementResourceValue<T> {\n    | ---------------------------------- doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`\n...\n131 |         let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_...\n    |                                                                  ^^^^^ function or associated item cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds\n    |\nnote: trait bound `T: std::cmp::PartialEq` was not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:55:14\n    |\n54  | impl<T> PCM for AgreementResourceValue<T>\n    |         ---     -------------------------\n55  |     where T: PartialEq\n    |              ^^^^^^^^^ unsatisfied trait bound introduced here\nhelp: consider restricting the type parameter to satisfy the trait bound\n    |\n42  | pub enum AgreementResourceValue<T> where T: std::cmp::PartialEq {\n    |                                    ++++++++++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"can't compare `T` with `T`","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":4086,"byte_end":4093,"line_start":139,"line_end":139,"column_start":35,"column_end":42,"is_primary":true,"text":[{"text":"        let tracked r = duplicate(&self.r);","highlight_start":35,"highlight_end":42}],"label":"no implementation for `T == T`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":4076,"byte_end":4085,"line_start":139,"line_end":139,"column_start":25,"column_end":34,"is_primary":false,"text":[{"text":"        let tracked r = duplicate(&self.r);","highlight_start":25,"highlight_end":34}],"label":"required by a bound introduced by this call","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required for `AgreementResourceValue<T>` to implement `vstd::pcm::PCM`","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1434,"byte_end":1443,"line_start":55,"line_end":55,"column_start":14,"column_end":23,"is_primary":false,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1387,"byte_end":1390,"line_start":54,"line_end":54,"column_start":9,"column_end":12,"is_primary":true,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1395,"byte_end":1420,"line_start":54,"line_end":54,"column_start":17,"column_end":42,"is_primary":true,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"required by a bound in `vstd::pcm_lib::duplicate`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/pcm_lib.rs","byte_start":2228,"byte_end":2314,"line_start":68,"line_end":68,"column_start":1,"column_end":87,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting type parameter `T`","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":3102,"byte_end":3102,"line_start":110,"line_end":110,"column_start":7,"column_end":7,"is_primary":true,"text":[{"text":"impl<T> AgreementResource<T> {","highlight_start":7,"highlight_end":7}],"label":null,"suggested_replacement":": std::cmp::PartialEq","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: can't compare `T` with `T`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:139:35\n    |\n139 |         let tracked r = duplicate(&self.r);\n    |                         --------- ^^^^^^^ no implementation for `T == T`\n    |                         |\n    |                         required by a bound introduced by this call\n    |\nnote: required for `AgreementResourceValue<T>` to implement `vstd::pcm::PCM`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:54:9\n    |\n54  | impl<T> PCM for AgreementResourceValue<T>\n    |         ^^^     ^^^^^^^^^^^^^^^^^^^^^^^^^\n55  |     where T: PartialEq\n    |              --------- unsatisfied trait bound introduced here\nnote: required by a bound in `vstd::pcm_lib::duplicate`\n   --> /Users/runner/work/verus/verus/source/vstd/pcm_lib.rs:68:1\nhelp: consider restricting type parameter `T`\n    |\n110 | impl<T: std::cmp::PartialEq> AgreementResource<T> {\n    |       +++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":4406,"byte_end":4416,"line_start":151,"line_end":151,"column_start":16,"column_end":26,"is_primary":true,"text":[{"text":"        self.r.validate_2(&other.r);","highlight_start":16,"highlight_end":26}],"label":"method cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1158,"byte_end":1192,"line_start":42,"line_end":42,"column_start":1,"column_end":35,"is_primary":false,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":1,"highlight_end":35}],"label":"doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"trait bound `T: std::cmp::PartialEq` was not satisfied","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1434,"byte_end":1443,"line_start":55,"line_end":55,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1387,"byte_end":1390,"line_start":54,"line_end":54,"column_start":9,"column_end":12,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1395,"byte_end":1420,"line_start":54,"line_end":54,"column_start":17,"column_end":42,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting the type parameter to satisfy the trait bound","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n","byte_start":1192,"byte_end":1192,"line_start":42,"line_end":42,"column_start":35,"column_end":35,"is_primary":true,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":35,"highlight_end":35}],"label":null,"suggested_replacement":" where T: std::cmp::PartialEq","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:151:16\n    |\n42  | pub enum AgreementResourceValue<T> {\n    | ---------------------------------- doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`\n...\n151 |         self.r.validate_2(&other.r);\n    |                ^^^^^^^^^^ method cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds\n    |\nnote: trait bound `T: std::cmp::PartialEq` was not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpessxd54n:55:14\n    |\n54  | impl<T> PCM for AgreementResourceValue<T>\n    |         ---     -------------------------\n55  |     where T: PartialEq\n    |              ^^^^^^^^^ unsatisfied trait bound introduced here\nhelp: consider restricting the type parameter to satisfy the trait bound\n    |\n42  | pub enum AgreementResourceValue<T> where T: std::cmp::PartialEq {\n    |                                    ++++++++++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 6 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 6 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0277, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0277, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0277`.\n"}
// 
// 