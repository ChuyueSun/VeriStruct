use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    match opt {
        MyOption::None => false,
        MyOption::Some(_) => true,
    }
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool {
    !is_Some(opt)
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A {
    match opt {
        MyOption::None => arbitrary(),
        MyOption::Some(a) => a,
    }
}

impl<A: Clone> Clone for MyOption<A> {
    fn clone(&self) -> Self {
        match self {
            MyOption::None => MyOption::None,
            MyOption::Some(a) => MyOption::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for MyOption<A> {
}

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A> {
        if is_Some(self) { self } else { optb }
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        ensures
            if is_None(self) {
                res == optb
            } else {
                res == self
            },
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res == is_Some(*self),
    {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res == is_None(*self),
    {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        ensures
            is_Some(*self) ==> (a == MyOption::Some(&get_Some_0(*self))),
            is_None(*self) ==> (a == MyOption::None),
    {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some(self),
        ensures
            a == get_Some_0(self),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some(self),
        ensures
            a == get_Some_0(self),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => proof_from_false(),
        }
    }
}

fn test_option_generic() {
    let opt: MyOption<i32> = MyOption::None;
    let is_none = opt.is_none();
    let is_some = opt.is_some();
    assert(is_none);
    assert(!is_some);

    let opt2: MyOption<i32> = MyOption::Some(42);
    let is_none = opt2.is_none();
    let is_some = opt2.is_some();
    assert(!is_none);
    assert(is_some);

    let opt3 = opt.or(opt2);
    let is_some = opt3.is_some();
    let val = opt3.unwrap();
    assert(is_some);
    assert(val == 42);

    let opt4 = opt2.or(opt);
    let is_some = opt4.is_some();
    let val = opt4.unwrap();
    assert(is_some);
    assert(val == 42);

    let opt5 = opt.or(MyOption::None);
    let is_none = opt5.is_none();
    let is_some = opt5.is_some();
    assert(is_none);
    assert(!is_some);
}

} // verus!

fn main() {
}
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.TypeAnnotation: type annotations needed
// {"$message_type":"diagnostic","message":"type annotations needed","code":{"code":"E0282","explanation":"The compiler could not infer a type and asked for a type annotation.\n\nErroneous code example:\n\n```compile_fail,E0282\nlet x = Vec::new();\n```\n\nThis error indicates that type inference did not result in one unique possible\ntype, and extra information is required. In most cases this can be provided\nby adding a type annotation. Sometimes you need to specify a generic type\nparameter manually.\n\nIn the example above, type `Vec` has a type parameter `T`. When calling\n`Vec::new`, barring any other later usage of the variable `x` that allows the\ncompiler to infer what type `T` is, the compiler needs to be told what it is.\n\nThe type can be specified on the variable:\n\n```\nlet x: Vec<i32> = Vec::new();\n```\n\nThe type can also be specified in the path of the expression:\n\n```\nlet x = Vec::<i32>::new();\n```\n\nIn cases with more complex types, it is not necessary to annotate the full\ntype. Once the ambiguity is resolved, the compiler can infer the rest:\n\n```\nlet x: Vec<_> = \"hello\".chars().rev().collect();\n```\n\nAnother way to provide the compiler with enough information, is to specify the\ngeneric type parameter:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<char>>();\n```\n\nAgain, you need not specify the full type if the compiler can infer it:\n\n```\nlet x = \"hello\".chars().rev().collect::<Vec<_>>();\n```\n\nApart from a method or function with a generic type parameter, this error can\noccur when a type parameter of a struct or trait cannot be inferred. In that\ncase it is not always possible to use a type annotation, because all candidates\nhave the same return type. For instance:\n\n```compile_fail,E0282\nstruct Foo<T> {\n    num: T,\n}\n\nimpl<T> Foo<T> {\n    fn bar() -> i32 {\n        0\n    }\n\n    fn baz() {\n        let number = Foo::bar();\n    }\n}\n```\n\nThis will fail because the compiler does not know which instance of `Foo` to\ncall `bar` on. Change `Foo::bar()` to `Foo::<T>::bar()` to resolve the error.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpx295_usq","byte_start":1917,"byte_end":1931,"line_start":87,"line_end":87,"column_start":38,"column_end":52,"is_primary":true,"text":[{"text":"            is_None(*self) ==> (a == MyOption::None),","highlight_start":38,"highlight_end":52}],"label":"cannot infer type of the type parameter `A` declared on the enum `MyOption`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider specifying the generic argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpx295_usq","byte_start":1925,"byte_end":1925,"line_start":87,"line_end":87,"column_start":46,"column_end":46,"is_primary":true,"text":[{"text":"            is_None(*self) ==> (a == MyOption::None),","highlight_start":46,"highlight_end":46}],"label":null,"suggested_replacement":"::<A>","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0282]: type annotations needed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpx295_usq:87:38\n   |\n87 |             is_None(*self) ==> (a == MyOption::None),\n   |                                      ^^^^^^^^^^^^^^ cannot infer type of the type parameter `A` declared on the enum `MyOption`\n   |\nhelp: consider specifying the generic argument\n   |\n87 |             is_None(*self) ==> (a == MyOption::<A>::None),\n   |                                              +++++\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0282`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0282`.\n"}
//
//
