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

impl<A: Copy> Copy for Option<A> { }

impl<A> Option<A> {
    pub open spec fn or(self, optb: Option<A>) -> Option<A> {
        match self {
            Option::None => optb,
            Option::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool) {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool) {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: Option<&A>) {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (a: A) {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A) {
        match self {
            Option::Some(a) => a,
            Option::None => proof_from_false(),
        }
    }
}

//----------------------------------------------------
// View trait implementation
//----------------------------------------------------
impl<A> View for Option<A> {
    type V = (bool, A);

    closed spec fn view(&self) -> Self::V {
        match self {
            Option::None => (false, arbitrary()),
            Option::Some(a) => (true, a),
        }
    }
}

} // verus!

fn main() {}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: cannot find trait `View` in this scope
// {"$message_type":"diagnostic","message":"cannot find trait `View` in this scope","code":{"code":"E0405","explanation":"The code refers to a trait that is not in scope.\n\nErroneous code example:\n\n```compile_fail,E0405\nstruct Foo;\n\nimpl SomeTrait for Foo {} // error: trait `SomeTrait` is not in scope\n```\n\nPlease verify that the name of the trait wasn't misspelled and ensure that it\nwas imported. Example:\n\n```\n# #[cfg(for_demonstration_only)]\n// solution 1:\nuse some_file::SomeTrait;\n\n// solution 2:\ntrait SomeTrait {\n    // some functions\n}\n\nstruct Foo;\n\nimpl SomeTrait for Foo { // ok!\n    // implements functions\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpww1rbyg6","byte_start":1948,"byte_end":1952,"line_start":89,"line_end":89,"column_start":9,"column_end":13,"is_primary":true,"text":[{"text":"impl<A> View for Option<A> {","highlight_start":9,"highlight_end":13}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider importing this trait","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpww1rbyg6","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::prelude::View;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0405]: cannot find trait `View` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpww1rbyg6:89:9\n   |\n89 | impl<A> View for Option<A> {\n   |         ^^^^ not found in this scope\n   |\nhelp: consider importing this trait\n   |\n1  + use vstd::prelude::View;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0405`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0405`.\n"}
// 
// 