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
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res <==> self.view().0,
    {
        match self {
            Option::Some(_) => true,
            Option::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res <==> !self.view().0,
    {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn as_ref(&self) -> (r: Option<&A>)
        ensures
            r.is_some() <==> self.view().0,
    {
        match self {
            Option::Some(x) => Option::Some(x),
            Option::None => Option::None,
        }
    }

    pub fn unwrap(self) -> (ret: A)
        requires
            self.view().0,
        ensures
            ret == self.view().1,
    {
        match self {
            Option::Some(a) => a,
            Option::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked ret: A)
        requires
            self.view().0,
        ensures
            ret == self.view().1,
    {
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
            Option::Some(a) => (true, ( a ) as A),
        }
    }
}

} // verus!

fn main() {
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: the trait bound `A: builtin::Integer` is not satisfied
// {"$message_type":"diagnostic","message":"the trait bound `A: builtin::Integer` is not satisfied","code":{"code":"E0277","explanation":"You tried to use a type which doesn't implement some trait in a place which\nexpected that trait.\n\nErroneous code example:\n\n```compile_fail,E0277\n// here we declare the Foo trait with a bar method\ntrait Foo {\n    fn bar(&self);\n}\n\n// we now declare a function which takes an object implementing the Foo trait\nfn some_func<T: Foo>(foo: T) {\n    foo.bar();\n}\n\nfn main() {\n    // we now call the method with the i32 type, which doesn't implement\n    // the Foo trait\n    some_func(5i32); // error: the trait bound `i32 : Foo` is not satisfied\n}\n```\n\nIn order to fix this error, verify that the type you're using does implement\nthe trait. Example:\n\n```\ntrait Foo {\n    fn bar(&self);\n}\n\n// we implement the trait on the i32 type\nimpl Foo for i32 {\n    fn bar(&self) {}\n}\n\nfn some_func<T: Foo>(foo: T) {\n    foo.bar(); // we can now use this method since i32 implements the\n               // Foo trait\n}\n\nfn main() {\n    some_func(5i32); // ok!\n}\n```\n\nOr in a generic context, an erroneous code example would look like:\n\n```compile_fail,E0277\nfn some_func<T>(foo: T) {\n    println!(\"{:?}\", foo); // error: the trait `core::fmt::Debug` is not\n                           //        implemented for the type `T`\n}\n\nfn main() {\n    // We now call the method with the i32 type,\n    // which *does* implement the Debug trait.\n    some_func(5i32);\n}\n```\n\nNote that the error here is in the definition of the generic function. Although\nwe only call it with a parameter that does implement `Debug`, the compiler\nstill rejects the function. It must work with all possible input types. In\norder to make this example compile, we need to restrict the generic type we're\naccepting:\n\n```\nuse std::fmt;\n\n// Restrict the input type to types that implement Debug.\nfn some_func<T: fmt::Debug>(foo: T) {\n    println!(\"{:?}\", foo);\n}\n\nfn main() {\n    // Calling the method is still fine, as i32 implements Debug.\n    some_func(5i32);\n\n    // This would fail to compile now:\n    // struct WithoutDebug;\n    // some_func(WithoutDebug);\n}\n```\n\nRust only looks at the signature of the called function, as such it must\nalready specify all requirements that will be used for every type parameter.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp79uc894l","byte_start":2469,"byte_end":2470,"line_start":115,"line_end":115,"column_start":48,"column_end":49,"is_primary":true,"text":[{"text":"            Option::Some(a) => (true, ( a ) as A),","highlight_start":48,"highlight_end":49}],"label":"the trait `builtin::Integer` is not implemented for `A`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"required by a bound in `builtin::spec_cast_integer`","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/builtin/src/lib.rs","byte_start":24453,"byte_end":24527,"line_start":934,"line_end":934,"column_start":1,"column_end":75,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"consider restricting type parameter `A`","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp79uc894l","byte_start":2259,"byte_end":2259,"line_start":109,"line_end":109,"column_start":7,"column_end":7,"is_primary":true,"text":[{"text":"impl<A> View for Option<A> {","highlight_start":7,"highlight_end":7}],"label":null,"suggested_replacement":": builtin::Integer","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0277]: the trait bound `A: builtin::Integer` is not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp79uc894l:115:48\n    |\n115 |             Option::Some(a) => (true, ( a ) as A),\n    |                                                ^ the trait `builtin::Integer` is not implemented for `A`\n    |\nnote: required by a bound in `builtin::spec_cast_integer`\n   --> /Users/runner/work/verus/verus/source/builtin/src/lib.rs:934:1\nhelp: consider restricting type parameter `A`\n    |\n109 | impl<A: builtin::Integer> View for Option<A> {\n    |       ++++++++++++++++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0277`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0277`.\n"}
// 
// 