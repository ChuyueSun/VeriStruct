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
        AgreementResourceValue::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T> {
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen { c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match self {
            AgreementResourceValue::Empty => other,
            AgreementResourceValue::Chosen { c } => match other {
                AgreementResourceValue::Empty => self,
                AgreementResourceValue::Chosen { c: d } => if c == d {
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

impl<T> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        true
    }

    pub closed spec fn id(self) -> Loc {
        arbitrary()
    }

    pub closed spec fn view(self) -> (Loc, T)
        recommends
            self.inv(),
    {
        (
            self.id(),
            match self.r@ {
                AgreementResourceValue::Chosen { c } => c,
                _ => arbitrary(),
            }
        )
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
        ensures
            result@ == c,
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result: AgreementResource<T>)
        ensures
            result@ == old(self)@,
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    ) -> (ret: ())
        requires
            old(self)@ == other@,
        ensures
            self@ == old(self)@,
            other@ == old(other)@,
    {
        self.r.validate_2(&other.r);
    }
}

impl<T> View for AgreementResource<T> {
    type V = T;

    closed spec fn view(&self) -> Self::V {
        AgreementResource::<T>::view(*self).1
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
// VerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.MismatchedType: mismatched typesVerusErrorType.TypeAnnotation: type annotations needed
// {"$message_type":"diagnostic","message":"unused import: `std::result::*`","code":{"code":"unused_imports","explanation":null},"level":"warning","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":43,"byte_end":57,"line_start":3,"line_end":3,"column_start":5,"column_end":19,"is_primary":true,"text":[{"text":"use std::result::*;","highlight_start":5,"highlight_end":19}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"`#[warn(unused_imports)]` on by default","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"remove the whole `use` item","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":39,"byte_end":59,"line_start":3,"line_end":4,"column_start":1,"column_end":1,"is_primary":true,"text":[{"text":"use std::result::*;","highlight_start":1,"highlight_end":20},{"text":"use vstd::pcm::*;","highlight_start":1,"highlight_end":1}],"label":null,"suggested_replacement":"","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"warning: unused import: `std::result::*`\n --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn:3:5\n  |\n3 | use std::result::*;\n  |     ^^^^^^^^^^^^^^\n  |\n  = note: `#[warn(unused_imports)]` on by default\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":1987,"byte_end":1988,"line_start":86,"line_end":86,"column_start":25,"column_end":26,"is_primary":true,"text":[{"text":"            match self.r@ {","highlight_start":25,"highlight_end":26}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn:86:25\n   |\n86 |             match self.r@ {\n   |                         ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":2945,"byte_end":2950,"line_start":118,"line_end":118,"column_start":27,"column_end":32,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":27,"highlight_end":32}],"label":"types differ in mutability","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":2941,"byte_end":2944,"line_start":118,"line_end":118,"column_start":23,"column_end":26,"is_primary":false,"text":[{"text":"            other@ == old(other)@,","highlight_start":23,"highlight_end":26}],"label":"arguments to this function are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"expected mutable reference `&mut _`\n           found reference `&AgreementResource<T>`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"function defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":6765,"byte_end":6768,"line_start":260,"line_end":260,"column_start":8,"column_end":11,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn:118:27\n    |\n118 |             other@ == old(other)@,\n    |                       --- ^^^^^ types differ in mutability\n    |                       |\n    |                       arguments to this function are incorrect\n    |\n    = note: expected mutable reference `&mut _`\n                       found reference `&AgreementResource<T>`\nnote: function defined here\n   --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:260:8\n\n"}
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":2951,"byte_end":2952,"line_start":118,"line_end":118,"column_start":33,"column_end":34,"is_primary":false,"text":[{"text":"            other@ == old(other)@,","highlight_start":33,"highlight_end":34}],"label":"type must be known at this point","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":2941,"byte_end":2944,"line_start":118,"line_end":118,"column_start":23,"column_end":26,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":23,"highlight_end":26}],"label":"cannot infer type of the type parameter `A` declared on the function `old`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn","byte_start":2944,"byte_end":2944,"line_start":118,"line_end":118,"column_start":26,"column_end":26,"is_primary":true,"text":[{"text":"            other@ == old(other)@,","highlight_start":26,"highlight_end":26}],"label":null,"suggested_replacement":"::<A>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpuygzcmnn:118:23\n    |\n118 |             other@ == old(other)@,\n    |                       ^^^       - type must be known at this point\n    |                       |\n    |                       cannot infer type of the type parameter `A` declared on the function `old`\n    |\nhelp: consider specifying the generic argument\n    |\n118 |             other@ == old::<A>(other)@,\n    |                          +++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors; 1 warning emitted","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors; 1 warning emitted\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0282, E0308, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0282, E0308, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0282`.\n"}
// 
// 