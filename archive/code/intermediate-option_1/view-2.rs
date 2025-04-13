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
            Option::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res == is_Some(*self),
    {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res == is_None(*self),
    {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: Option<&A>)
        ensures
            is_Some(a) == is_Some(*self),
    {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some(*self),
        ensures
            a == get_Some_0(*self),
    {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some(*self),
        ensures
            a == get_Some_0(*self),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: type `Option<A>` cannot be dereferencedVerusErrorType.Other: type `Option<A>` cannot be dereferencedVerusErrorType.Other: type `Option<A>` cannot be dereferencedVerusErrorType.Other: type `Option<A>` cannot be dereferenced
// {"$message_type":"diagnostic","message":"type `Option<A>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4","byte_start":1709,"byte_end":1714,"line_start":85,"line_end":85,"column_start":21,"column_end":26,"is_primary":true,"text":[{"text":"            is_Some(*self),","highlight_start":21,"highlight_end":26}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `Option<A>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4:85:21\n   |\n85 |             is_Some(*self),\n   |                     ^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"type `Option<A>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4","byte_start":1761,"byte_end":1766,"line_start":87,"line_end":87,"column_start":29,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(*self),","highlight_start":29,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `Option<A>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4:87:29\n   |\n87 |             a == get_Some_0(*self),\n   |                             ^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"type `Option<A>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4","byte_start":1989,"byte_end":1994,"line_start":97,"line_end":97,"column_start":21,"column_end":26,"is_primary":true,"text":[{"text":"            is_Some(*self),","highlight_start":21,"highlight_end":26}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `Option<A>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4:97:21\n   |\n97 |             is_Some(*self),\n   |                     ^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"type `Option<A>` cannot be dereferenced","code":{"code":"E0614","explanation":"Attempted to dereference a variable which cannot be dereferenced.\n\nErroneous code example:\n\n```compile_fail,E0614\nlet y = 0u32;\n*y; // error: type `u32` cannot be dereferenced\n```\n\nOnly types implementing `std::ops::Deref` can be dereferenced (such as `&T`).\nExample:\n\n```\nlet y = 0u32;\nlet x = &y;\n// So here, `x` is a `&u32`, so we can dereference it:\n*x; // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4","byte_start":2041,"byte_end":2046,"line_start":99,"line_end":99,"column_start":29,"column_end":34,"is_primary":true,"text":[{"text":"            a == get_Some_0(*self),","highlight_start":29,"highlight_end":34}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0614]: type `Option<A>` cannot be dereferenced\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpej4as3b4:99:29\n   |\n99 |             a == get_Some_0(*self),\n   |                             ^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0614`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0614`.\n"}
// 
// 