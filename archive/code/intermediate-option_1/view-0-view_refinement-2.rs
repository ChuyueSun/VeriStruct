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

pub open spec fn get_Some_0<A>(opt: Option<A>) -> A {
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

pub trait View {
    type V;
    closed spec fn view(&self) -> Self::V;
}

impl<A> View for Option<A> {
    type V = (bool, A);

    closed spec fn view(&self) -> Self::V {
        match self {
            Option::None => (false, arbitrary()),
            Option::Some(a) => (true, *a),
        }
    }
}

} // verus!

fn main() {
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: trait function declarations cannot be open or closed, as they don't have a body
// {"$message_type":"diagnostic","message":"trait function declarations cannot be open or closed, as they don't have a body","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfsxxq4n5","byte_start":1846,"byte_end":1871,"line_start":87,"line_end":87,"column_start":17,"column_end":42,"is_primary":true,"text":[{"text":"    closed spec fn view(&self) -> Self::V;","highlight_start":17,"highlight_end":42}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: trait function declarations cannot be open or closed, as they don't have a body\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfsxxq4n5:87:17\n   |\n87 |     closed spec fn view(&self) -> Self::V;\n   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 