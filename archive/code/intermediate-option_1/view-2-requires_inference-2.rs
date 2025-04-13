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
        ensures
            res == is_None(self@),
    {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: Option<&A>)
        ensures
            if is_Some(self@) {
                is_Some(a)
            } else {
                is_None(a)
            },
    {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        ensures
            a == get_Some_0(self@),
    {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 5
// VerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for reference `&Option<A>` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scopeVerusErrorType.Other: no method named `view` found for enum `Option` in the current scope
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":1046,"byte_end":1047,"line_start":54,"line_end":54,"column_start":32,"column_end":33,"is_primary":true,"text":[{"text":"            res == is_Some(self@),","highlight_start":32,"highlight_end":33}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj:54:32\n   |\n54 |             res == is_Some(self@),\n   |                                ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":1282,"byte_end":1283,"line_start":65,"line_end":65,"column_start":32,"column_end":33,"is_primary":true,"text":[{"text":"            res == is_None(self@),","highlight_start":32,"highlight_end":33}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj:65:32\n   |\n65 |             res == is_None(self@),\n   |                                ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for reference `&Option<A>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":1489,"byte_end":1490,"line_start":75,"line_end":75,"column_start":28,"column_end":29,"is_primary":true,"text":[{"text":"            if is_Some(self@) {","highlight_start":28,"highlight_end":29}],"label":"method not found in `&Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for reference `&Option<A>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj:75:28\n   |\n75 |             if is_Some(self@) {\n   |                            ^ method not found in `&Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":1800,"byte_end":1801,"line_start":89,"line_end":89,"column_start":33,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(self@),","highlight_start":33,"highlight_end":34}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj:89:33\n   |\n8  | pub enum Option<A> {\n   | ------------------ method `view` not found for this enum\n...\n89 |             a == get_Some_0(self@),\n   |                                 ^ method not found in `Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `view` found for enum `Option` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":2035,"byte_end":2036,"line_start":99,"line_end":99,"column_start":33,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(self@),","highlight_start":33,"highlight_end":34}],"label":"method not found in `Option<A>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj","byte_start":120,"byte_end":138,"line_start":8,"line_end":8,"column_start":1,"column_end":19,"is_primary":false,"text":[{"text":"pub enum Option<A> {","highlight_start":1,"highlight_end":19}],"label":"method `view` not found for this enum","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"items from traits can only be used if the trait is implemented and in scope","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"the following traits define an item `view`, perhaps you need to implement one of them:\ncandidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\ncandidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\ncandidate #3: `vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `view` found for enum `Option` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp5aglsioj:99:33\n   |\n8  | pub enum Option<A> {\n   | ------------------ method `view` not found for this enum\n...\n99 |             a == get_Some_0(self@),\n   |                                 ^ method not found in `Option<A>`\n   |\n   = help: items from traits can only be used if the trait is implemented and in scope\n   = note: the following traits define an item `view`, perhaps you need to implement one of them:\n           candidate #1: `vstd::std_specs::hash::IterAdditionalSpecFns`\n           candidate #2: `vstd::std_specs::hash::KeysAdditionalSpecFns`\n           candidate #3: `vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 5 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 5 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 