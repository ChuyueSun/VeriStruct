#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

#[doc = r#"
This file implements agreement on a constant value using a custom
resource algebra.

An agreement resource constitutes knowledge of a constant value.
To create an instance of a constant value of type `T`, use
`AgreementResource::<T>::alloc()` as in the following example:

let tracked r1 = AgreementResource::<int>::alloc(72);
assert(r1@ == 72);

Knowledge of a constant value can be duplicated with `duplicate`,
which creates another agreement resource with the same constant
value and the same ID. Here's an example:

let tracked r2 = r1.duplicate();
assert(r2.id() == r1.id());
assert(r2@ == r1@);

Any two agreement resources with the same `id()` are guaranteed to
have equal values. You can establish this by calling
`lemma_agreement`, as in the following example:

assert(r2.id() == r1.id());
proof { r1.lemma_agreement(&mut r2); }
assert(r2@ == r1@);
"#]

verus! {

#[verifier::loop_isolation(false)]
pub enum AgreementResourceValue<T> {
    Empty,
    Chosen { c: T },
    Invalid,
}

impl<T> AgreementResourceValue<T> {
    pub open spec fn new(c: T) -> Self {
        AgreementResourceValue::Chosen { c }
        /* TODO: part of view */
    }
}

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Invalid => false,
            _ => true,
        }
        /* TODO: part of view */
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Invalid => AgreementResourceValue::Invalid,
                AgreementResourceValue::Chosen { c: c2 } => if c == c2 {
                    AgreementResourceValue::Chosen { c }
                } else {
                    AgreementResourceValue::Invalid
                },
                AgreementResourceValue::Empty => AgreementResourceValue::Chosen { c },
            },
            AgreementResourceValue::Empty => other,
        }
        /* TODO: part of view */
    }

    open spec fn unit() -> Self {
        AgreementResourceValue::Empty
        /* TODO: part of view */
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
        self.r@.valid()
        /* TODO: part of view */
    }

    pub closed spec fn id(self) -> Loc {
        self.r.id()
        /* TODO: part of view */
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        match self.r@ {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
        /* TODO: part of view */
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
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
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.inv(),
            self.id() == old(self).id(),
            result.id() == old(self).id(),
            self@ == old(self)@,
            result@ == old(self)@,
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
        requires
            old(self).inv(),
            old(other).inv(),
        ensures
            self.inv(),
            other.inv(),
            self@ == old(self)@,
            other@ == old(other)@,
            if self.id() == other.id() { self@ == other@ } else { true },
    {
        self.r.validate_2(&other.r);
    }
}

impl<T> View for AgreementResource<T> {
    type V = (Loc, Option<T>);

    closed spec fn view(&self) -> Self::V {
        let val = match self.r@ {
            AgreementResourceValue::Chosen { c } => Some(c),
            _ => None,
        };
        (self.id(), val)
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 8
// VerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `id` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.MismatchedType: mismatched typesVerusErrorType.TypeAnnotation: type annotations neededVerusErrorType.MismatchedType: mismatched typesVerusErrorType.TypeAnnotation: type annotations neededVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scope
// {"$message_type":"diagnostic","message":"unused attribute `doc`","code":{"code":"unused_attributes","explanation":null},"level":"warning","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":148,"byte_end":1021,"line_start":9,"line_end":35,"column_start":1,"column_end":4,"is_primary":true,"text":[{"text":"#[doc = r#\"","highlight_start":1,"highlight_end":12},{"text":"This file implements agreement on a constant value using a custom","highlight_start":1,"highlight_end":66},{"text":"resource algebra.","highlight_start":1,"highlight_end":18},{"text":"","highlight_start":1,"highlight_end":1},{"text":"An agreement resource constitutes knowledge of a constant value.","highlight_start":1,"highlight_end":65},{"text":"To create an instance of a constant value of type `T`, use","highlight_start":1,"highlight_end":59},{"text":"`AgreementResource::<T>::alloc()` as in the following example:","highlight_start":1,"highlight_end":63},{"text":"","highlight_start":1,"highlight_end":1},{"text":"let tracked r1 = AgreementResource::<int>::alloc(72);","highlight_start":1,"highlight_end":54},{"text":"assert(r1@ == 72);","highlight_start":1,"highlight_end":19},{"text":"","highlight_start":1,"highlight_end":1},{"text":"Knowledge of a constant value can be duplicated with `duplicate`,","highlight_start":1,"highlight_end":66},{"text":"which creates another agreement resource with the same constant","highlight_start":1,"highlight_end":64},{"text":"value and the same ID. Here's an example:","highlight_start":1,"highlight_end":42},{"text":"","highlight_start":1,"highlight_end":1},{"text":"let tracked r2 = r1.duplicate();","highlight_start":1,"highlight_end":33},{"text":"assert(r2.id() == r1.id());","highlight_start":1,"highlight_end":28},{"text":"assert(r2@ == r1@);","highlight_start":1,"highlight_end":20},{"text":"","highlight_start":1,"highlight_end":1},{"text":"Any two agreement resources with the same `id()` are guaranteed to","highlight_start":1,"highlight_end":67},{"text":"have equal values. You can establish this by calling","highlight_start":1,"highlight_end":53},{"text":"`lemma_agreement`, as in the following example:","highlight_start":1,"highlight_end":48},{"text":"","highlight_start":1,"highlight_end":1},{"text":"assert(r2.id() == r1.id());","highlight_start":1,"highlight_end":28},{"text":"proof { r1.lemma_agreement(&mut r2); }","highlight_start":1,"highlight_end":39},{"text":"assert(r2@ == r1@);","highlight_start":1,"highlight_end":20},{"text":"\"#]","highlight_start":1,"highlight_end":4}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the built-in attribute `doc` will be ignored, since it's applied to the macro invocation `verus`","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":1023,"byte_end":1028,"line_start":37,"line_end":37,"column_start":1,"column_end":6,"is_primary":true,"text":[{"text":"verus! {","highlight_start":1,"highlight_end":6}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"`#[warn(unused_attributes)]` on by default","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"warning: unused attribute `doc`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:9:1\n   |\n9  | / #[doc = r#\"\n10 | | This file implements agreement on a constant value using a custom\n11 | | resource algebra.\n...  |\n34 | | assert(r2@ == r1@);\n35 | | \"#]\n   | |___^\n   |\nnote: the built-in attribute `doc` will be ignored, since it's applied to the macro invocation `verus`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:37:1\n   |\n37 | verus! {\n   | ^^^^^\n   = note: `#[warn(unused_attributes)]` on by default\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":2971,"byte_end":2972,"line_start":111,"line_end":111,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@.valid()","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:111:15\n    |\n111 |         self.r@.valid()\n    |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `id` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":3077,"byte_end":3079,"line_start":116,"line_end":116,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        self.r.id()","highlight_start":16,"highlight_end":18}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `id` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:116:16\n    |\n116 |         self.r.id()\n    |                ^^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":3230,"byte_end":3231,"line_start":124,"line_end":124,"column_start":21,"column_end":22,"is_primary":true,"text":[{"text":"        match self.r@ {","highlight_start":21,"highlight_end":22}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:124:21\n    |\n124 |         match self.r@ {\n    |                     ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4398,"byte_end":4403,"line_start":164,"line_end":164,"column_start":17,"column_end":22,"is_primary":true,"text":[{"text":"            old(other).inv(),","highlight_start":17,"highlight_end":22}],"label":"types differ in mutability","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4394,"byte_end":4397,"line_start":164,"line_end":164,"column_start":13,"column_end":16,"is_primary":false,"text":[{"text":"            old(other).inv(),","highlight_start":13,"highlight_end":16}],"label":"arguments to this function are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"expected mutable reference `&mut _`\n           found reference `&AgreementResource<T>`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"function defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":6765,"byte_end":6768,"line_start":260,"line_end":260,"column_start":8,"column_end":11,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:164:17\n    |\n164 |             old(other).inv(),\n    |             --- ^^^^^ types differ in mutability\n    |             |\n    |             arguments to this function are incorrect\n    |\n    = note: expected mutable reference `&mut _`\n                       found reference `&AgreementResource<T>`\nnote: function defined here\n   --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:260:8\n\n"}
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4405,"byte_end":4408,"line_start":164,"line_end":164,"column_start":24,"column_end":27,"is_primary":false,"text":[{"text":"            old(other).inv(),","highlight_start":24,"highlight_end":27}],"label":"type must be known at this point","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4394,"byte_end":4397,"line_start":164,"line_end":164,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"            old(other).inv(),","highlight_start":13,"highlight_end":16}],"label":"cannot infer type of the type parameter `A` declared on the function `old`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4397,"byte_end":4397,"line_start":164,"line_end":164,"column_start":16,"column_end":16,"is_primary":true,"text":[{"text":"            old(other).inv(),","highlight_start":16,"highlight_end":16}],"label":null,"suggested_replacement":"::<A>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:164:13\n    |\n164 |             old(other).inv(),\n    |             ^^^        --- type must be known at this point\n    |             |\n    |             cannot infer type of the type parameter `A` declared on the function `old`\n    |\nhelp: consider specifying the generic argument\n    |\n164 |             old::<A>(other).inv(),\n    |                +++++\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4536,"byte_end":4541,"line_start":169,"line_end":169,"column_start":27,"column_end":32,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":27,"highlight_end":32}],"label":"types differ in mutability","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4532,"byte_end":4535,"line_start":169,"line_end":169,"column_start":23,"column_end":26,"is_primary":false,"text":[{"text":"            other@ == old(other)@,","highlight_start":23,"highlight_end":26}],"label":"arguments to this function are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"expected mutable reference `&mut _`\n           found reference `&AgreementResource<T>`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"function defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":6765,"byte_end":6768,"line_start":260,"line_end":260,"column_start":8,"column_end":11,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:169:27\n    |\n169 |             other@ == old(other)@,\n    |                       --- ^^^^^ types differ in mutability\n    |                       |\n    |                       arguments to this function are incorrect\n    |\n    = note: expected mutable reference `&mut _`\n                       found reference `&AgreementResource<T>`\nnote: function defined here\n   --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:260:8\n\n"}
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4542,"byte_end":4543,"line_start":169,"line_end":169,"column_start":33,"column_end":34,"is_primary":false,"text":[{"text":"            other@ == old(other)@,","highlight_start":33,"highlight_end":34}],"label":"type must be known at this point","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4532,"byte_end":4535,"line_start":169,"line_end":169,"column_start":23,"column_end":26,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":23,"highlight_end":26}],"label":"cannot infer type of the type parameter `A` declared on the function `old`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4535,"byte_end":4535,"line_start":169,"line_end":169,"column_start":26,"column_end":26,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":26,"highlight_end":26}],"label":null,"suggested_replacement":"::<A>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:169:23\n    |\n169 |             other@ == old(other)@,\n    |                       ^^^       - type must be known at this point\n    |                       |\n    |                       cannot infer type of the type parameter `A` declared on the function `old`\n    |\nhelp: consider specifying the generic argument\n    |\n169 |             other@ == old::<A>(other)@,\n    |                          +++++\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn","byte_start":4817,"byte_end":4818,"line_start":180,"line_end":180,"column_start":31,"column_end":32,"is_primary":true,"text":[{"text":"        let val = match self.r@ {","highlight_start":31,"highlight_end":32}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpph6se3cn:180:31\n    |\n180 |         let val = match self.r@ {\n    |                               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 8 previous errors; 1 warning emitted","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 8 previous errors; 1 warning emitted\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0282, E0308, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0282, E0308, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0282`.\n"}
//
//
