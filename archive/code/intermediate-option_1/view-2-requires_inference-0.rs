use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum Option<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: Option<A>) -> bool {
    matches!(opt, Option::Some(_))
}

pub open spec fn is_None<A>(opt: Option<A>) -> bool {
    matches!(opt, Option::None)
}

pub open spec fn get_Some_0<A>(opt: Option<A>) -> A
{
    match opt {
        Option::Some(a) => a,
        Option::None => arbitrary(),
    }
}


impl<A: Clone> Clone for Option<A> {
    fn clone(&self) -> Self {
        match self {
            Option::None => Option::None,
            Option::Some(a) => Option::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for Option<A> {

}

impl<A> Option<A> {
    pub open spec fn or(self, optb: Option<A>) -> Option<A> {
        match self {
            Option::None => optb,
            Option::Some(s) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        requires
            true,
        ensures
            res == is_Some(self@),
    {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        requires
            true,
        ensures
            res == is_None(self@),
    {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: Option<&A>)
        requires
            true,
        ensures
            is_Some(self@) <==> is_Some(a@),
    {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(self@),
    {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(self@),
    {
        match self {
            Option::Some(a) => a,
            Option::None => proof_from_false(),
        }
    }
}

} // verus!

fn main() {
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 8
// VerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scope
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1081,"byte_end":1082,"line_start":56,"line_end":56,"column_start":32,"column_end":33,"is_primary":true,"text":[{"text":"            res == is_Some(self@),","highlight_start":32,"highlight_end":33}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:56:32\n   |\n56 |             res == is_Some(self@),\n   |                                ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1352,"byte_end":1353,"line_start":69,"line_end":69,"column_start":32,"column_end":33,"is_primary":true,"text":[{"text":"            res == is_None(self@),","highlight_start":32,"highlight_end":33}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:69:32\n   |\n69 |             res == is_None(self@),\n   |                                ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1591,"byte_end":1592,"line_start":81,"line_end":81,"column_start":25,"column_end":26,"is_primary":true,"text":[{"text":"            is_Some(self@) <==> is_Some(a@),","highlight_start":25,"highlight_end":26}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:81:25\n   |\n81 |             is_Some(self@) <==> is_Some(a@),\n   |                         ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1608,"byte_end":1609,"line_start":81,"line_end":81,"column_start":42,"column_end":43,"is_primary":true,"text":[{"text":"            is_Some(self@) <==> is_Some(a@),","highlight_start":42,"highlight_end":43}],"label":"method not found in `Option<&A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:81:42\n   |\n8  | pub enum Option<A> {\n   | ------------------ method `view` not found for this enum\n...\n81 |             is_Some(self@) <==> is_Some(a@),\n   |                                          ^ method not found in `Option<&A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1821,"byte_end":1822,"line_start":91,"line_end":91,"column_start":25,"column_end":26,"is_primary":true,"text":[{"text":"            is_Some(self@),","highlight_start":25,"highlight_end":26}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:91:25\n   |\n8  | pub enum Option<A> {\n   | ------------------ method `view` not found for this enum\n...\n91 |             is_Some(self@),\n   |                         ^ method not found in `Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":1873,"byte_end":1874,"line_start":93,"line_end":93,"column_start":33,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(self@),","highlight_start":33,"highlight_end":34}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:93:33\n   |\n8  | pub enum Option<A> {\n   | ------------------ method `view` not found for this enum\n...\n93 |             a == get_Some_0(self@),\n   |                                 ^ method not found in `Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":2101,"byte_end":2102,"line_start":103,"line_end":103,"column_start":25,"column_end":26,"is_primary":true,"text":[{"text":"            is_Some(self@),","highlight_start":25,"highlight_end":26}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:103:25\n    |\n8   | pub enum Option<A> {\n    | ------------------ method `view` not found for this enum\n...\n103 |             is_Some(self@),\n    |                         ^ method not found in `Option<A>`\n    |\n    = help: items from traits can only be used if the trait is implemented and in scope\n    = note: the following traits define an item `view`, perhaps you need to implement one of them:\n            candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n            candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n            candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":2153,"byte_end":2154,"line_start":105,"line_end":105,"column_start":33,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(self@),","highlight_start":33,"highlight_end":34}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp1tyntotu:105:33\n    |\n8   | pub enum Option<A> {\n    | ------------------ method `view` not found for this enum\n...\n105 |             a == get_Some_0(self@),\n    |                                 ^ method not found in `Option<A>`\n    |\n    = help: items from traits can only be used if the trait is implemented and in scope\n    = note: the following traits define an item `view`, perhaps you need to implement one of them:\n            candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n            candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n            candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 8 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 8 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
//
//
