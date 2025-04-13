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
        AgreementResourceValue::Chosen { c }
    }
}

impl<T> PCM for AgreementResourceValue<T>
    where T: PartialEq
{
    open spec fn valid(self) -> bool {
        match self {
            AgreementResourceValue::Empty => true,
            AgreementResourceValue::Chosen{ c: _ } => true,
            AgreementResourceValue::Invalid => false,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (AgreementResourceValue::Invalid, _) => AgreementResourceValue::Invalid,
            (_, AgreementResourceValue::Invalid) => AgreementResourceValue::Invalid,
            (AgreementResourceValue::Empty, x) => x,
            (x, AgreementResourceValue::Empty) => x,
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

    proof fn closed_under_incl(a: Self, b: Self) {
        // Trivial for agreement-based resource.
    }

    proof fn commutative(a: Self, b: Self) {
        // op(a, b) = op(b, a) by construction, done by pattern matching.
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        // op(op(a, b), c) = op(a, op(b, c)) by construction, done by pattern matching.
    }

    proof fn op_unit(a: Self) {
        // op(a, unit()) = a, op(unit(), a) = a.
    }

    proof fn unit_valid() {
        // unit() is valid.
    }
}

pub struct AgreementResource<T> {
    r: Resource<AgreementResourceValue<T>>,
}

impl<T> AgreementResource<T> {
    pub closed spec fn inv(self) -> bool {
        self.r@.value.valid()
    }

    pub closed spec fn id(self) -> Loc {
        self.r@.id
    }

    pub closed spec fn view(self) -> T
        recommends
            self.inv(),
    {
        match self.r@.value {
            AgreementResourceValue::Chosen { c } => c,
            _ => arbitrary(),
        }
    }

    pub proof fn alloc(c: T) -> (tracked result: AgreementResource<T>)
    // No extra conditions required, vault is always consistent upon allocation
    {
        let r_value = AgreementResourceValue::<T>::new(c);
        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);
        AgreementResource::<T> { r }
    }

    pub proof fn duplicate(tracked self: &mut AgreementResource<T>) -> (tracked result:
        AgreementResource<T>)
    // The resource can always be duplicated
    {
        let tracked r = duplicate(&self.r);
        AgreementResource::<T> { r }
    }

    pub proof fn lemma_agreement(
        tracked self: &mut AgreementResource<T>,
        tracked other: &AgreementResource<T>,
    )
    // If the IDs match, the values are forced to match via `validate_2`
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 6
// VerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: no method named `view` found for struct `vstd::pcm::Resource` in the current scopeVerusErrorType.Other: the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfiedVerusErrorType.Other: can't compare `T` with `T`VerusErrorType.Other: the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2140,"byte_end":2141,"line_start":81,"line_end":81,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@.value.valid()","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:81:15\n   |\n81 |         self.r@.value.valid()\n   |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2218,"byte_end":2219,"line_start":85,"line_end":85,"column_start":15,"column_end":16,"is_primary":true,"text":[{"text":"        self.r@.id","highlight_start":15,"highlight_end":16}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:85:15\n   |\n85 |         self.r@.id\n   |               ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for struct `vstd::pcm::Resource` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2338,"byte_end":2339,"line_start":92,"line_end":92,"column_start":21,"column_end":22,"is_primary":true,"text":[{"text":"        match self.r@.value {","highlight_start":21,"highlight_end":22}],"label":"method not found in `Resource<AgreementResourceValue<T>>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `view` found for struct `vstd::pcm::Resource` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:92:21\n   |\n92 |         match self.r@.value {\n   |                     ^ method not found in `Resource<AgreementResourceValue<T>>`\n\n"}
// {"$message_type":"diagnostic","message":"the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2731,"byte_end":2736,"line_start":102,"line_end":102,"column_start":66,"column_end":71,"is_primary":true,"text":[{"text":"        let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_value);","highlight_start":66,"highlight_end":71}],"label":"function or associated item cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":158,"byte_end":192,"line_start":11,"line_end":11,"column_start":1,"column_end":35,"is_primary":false,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":1,"highlight_end":35}],"label":"doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"trait bound `T: std::cmp::PartialEq` was not satisfied","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":429,"byte_end":438,"line_start":24,"line_end":24,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":382,"byte_end":385,"line_start":23,"line_end":23,"column_start":9,"column_end":12,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":390,"byte_end":415,"line_start":23,"line_end":23,"column_start":17,"column_end":42,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting the type parameter to satisfy the trait bound","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":192,"byte_end":192,"line_start":11,"line_end":11,"column_start":35,"column_end":35,"is_primary":true,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":35,"highlight_end":35}],"label":null,"suggested_replacement":" where T: std::cmp::PartialEq","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: the function or associated item `alloc` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:102:66\n    |\n11  | pub enum AgreementResourceValue<T> {\n    | ---------------------------------- doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`\n...\n102 |         let tracked r = Resource::<AgreementResourceValue::<T>>::alloc(r_...\n    |                                                                  ^^^^^ function or associated item cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds\n    |\nnote: trait bound `T: std::cmp::PartialEq` was not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:24:14\n    |\n23  | impl<T> PCM for AgreementResourceValue<T>\n    |         ---     -------------------------\n24  |     where T: PartialEq\n    |              ^^^^^^^^^ unsatisfied trait bound introduced here\nhelp: consider restricting the type parameter to satisfy the trait bound\n    |\n11  | pub enum AgreementResourceValue<T> where T: std::cmp::PartialEq {\n    |                                    ++++++++++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"can't compare `T` with `T`","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2994,"byte_end":3001,"line_start":110,"line_end":110,"column_start":35,"column_end":42,"is_primary":true,"text":[{"text":"        let tracked r = duplicate(&self.r);","highlight_start":35,"highlight_end":42}],"label":"no implementation for `T == T`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2984,"byte_end":2993,"line_start":110,"line_end":110,"column_start":25,"column_end":34,"is_primary":false,"text":[{"text":"        let tracked r = duplicate(&self.r);","highlight_start":25,"highlight_end":34}],"label":"required by a bound introduced by this call","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required for `AgreementResourceValue<T>` to implement `vstd::pcm::PCM`","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":429,"byte_end":438,"line_start":24,"line_end":24,"column_start":14,"column_end":23,"is_primary":false,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":382,"byte_end":385,"line_start":23,"line_end":23,"column_start":9,"column_end":12,"is_primary":true,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":390,"byte_end":415,"line_start":23,"line_end":23,"column_start":17,"column_end":42,"is_primary":true,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"required by a bound in `vstd::pcm_lib::duplicate`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/pcm_lib.rs","byte_start":2228,"byte_end":2314,"line_start":68,"line_end":68,"column_start":1,"column_end":87,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting type parameter `T`","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":2058,"byte_end":2058,"line_start":79,"line_end":79,"column_start":7,"column_end":7,"is_primary":true,"text":[{"text":"impl<T> AgreementResource<T> {","highlight_start":7,"highlight_end":7}],"label":null,"suggested_replacement":": std::cmp::PartialEq","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: can't compare `T` with `T`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:110:35\n    |\n110 |         let tracked r = duplicate(&self.r);\n    |                         --------- ^^^^^^^ no implementation for `T == T`\n    |                         |\n    |                         required by a bound introduced by this call\n    |\nnote: required for `AgreementResourceValue<T>` to implement `vstd::pcm::PCM`\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:23:9\n    |\n23  | impl<T> PCM for AgreementResourceValue<T>\n    |         ^^^     ^^^^^^^^^^^^^^^^^^^^^^^^^\n24  |     where T: PartialEq\n    |              --------- unsatisfied trait bound introduced here\nnote: required by a bound in `vstd::pcm_lib::duplicate`\n   --> /Users/runner/work/verus/verus/source/vstd/pcm_lib.rs:68:1\nhelp: consider restricting type parameter `T`\n    |\n79  | impl<T: std::cmp::PartialEq> AgreementResource<T> {\n    |       +++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":3277,"byte_end":3287,"line_start":120,"line_end":120,"column_start":16,"column_end":26,"is_primary":true,"text":[{"text":"        self.r.validate_2(&other.r);","highlight_start":16,"highlight_end":26}],"label":"method cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":158,"byte_end":192,"line_start":11,"line_end":11,"column_start":1,"column_end":35,"is_primary":false,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":1,"highlight_end":35}],"label":"doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"trait bound `T: std::cmp::PartialEq` was not satisfied","code":null,"level":"note","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":429,"byte_end":438,"line_start":24,"line_end":24,"column_start":14,"column_end":23,"is_primary":true,"text":[{"text":"    where T: PartialEq","highlight_start":14,"highlight_end":23}],"label":"unsatisfied trait bound introduced here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":382,"byte_end":385,"line_start":23,"line_end":23,"column_start":9,"column_end":12,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":9,"highlight_end":12}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":390,"byte_end":415,"line_start":23,"line_end":23,"column_start":17,"column_end":42,"is_primary":false,"text":[{"text":"impl<T> PCM for AgreementResourceValue<T>","highlight_start":17,"highlight_end":42}],"label":"","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting the type parameter to satisfy the trait bound","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u","byte_start":192,"byte_end":192,"line_start":11,"line_end":11,"column_start":35,"column_end":35,"is_primary":true,"text":[{"text":"pub enum AgreementResourceValue<T> {","highlight_start":35,"highlight_end":35}],"label":null,"suggested_replacement":" where T: std::cmp::PartialEq","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: the method `validate_2` exists for struct `Resource<AgreementResourceValue<T>>`, but its trait bounds were not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:120:16\n    |\n11  | pub enum AgreementResourceValue<T> {\n    | ---------------------------------- doesn't satisfy `AgreementResourceValue<T>: vstd::pcm::PCM`\n...\n120 |         self.r.validate_2(&other.r);\n    |                ^^^^^^^^^^ method cannot be called on `Resource<AgreementResourceValue<T>>` due to unsatisfied trait bounds\n    |\nnote: trait bound `T: std::cmp::PartialEq` was not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmph3bbtl2u:24:14\n    |\n23  | impl<T> PCM for AgreementResourceValue<T>\n    |         ---     -------------------------\n24  |     where T: PartialEq\n    |              ^^^^^^^^^ unsatisfied trait bound introduced here\nhelp: consider restricting the type parameter to satisfy the trait bound\n    |\n11  | pub enum AgreementResourceValue<T> where T: std::cmp::PartialEq {\n    |                                    ++++++++++++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 6 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 6 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0277, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0277, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0277`.\n"}
// 
// 