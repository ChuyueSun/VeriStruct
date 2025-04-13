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
    match opt {
        MyOption::None => true,
        MyOption::Some(_) => false,
    }
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A {
    match opt {
        MyOption::Some(a) => a,
        MyOption::None => unreachable(),
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
        // TODO: add specification
        self // placeholder; actual logic not required here as it's an open spec
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        requires
            true,
        ensures
            is_None(self@) ==> (res@ == optb@),
            !is_None(self@) ==> (res@ == self@),
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
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
            MyOption::Some(_) => true,
            MyOption::None => false,
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
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        requires
            true,
        ensures
            is_None(self@) ==> is_None(a@),
            is_Some(self@) ==> is_Some(a@),
    {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> (a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(self@),
    {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreached(),
        }
    }

    pub proof fn tracked_unwrap(tracked self) -> (tracked a: A)
        requires
            is_Some(self@),
        ensures
            a == get_Some_0(self@),
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
// VerusErrorType.Other: expected function, found macro `unreachable`
// {"$message_type":"diagnostic","message":"expected function, found macro `unreachable`","code":{"code":"E0423","explanation":"An identifier was used like a function name or a value was expected and the\nidentifier exists but it belongs to a different namespace.\n\nErroneous code example:\n\n```compile_fail,E0423\nstruct Foo { a: bool };\n\nlet f = Foo();\n// error: expected function, tuple struct or tuple variant, found `Foo`\n// `Foo` is a struct name, but this expression uses it like a function name\n```\n\nPlease verify you didn't misspell the name of what you actually wanted to use\nhere. Example:\n\n```\nfn Foo() -> u32 { 0 }\n\nlet f = Foo(); // ok!\n```\n\nIt is common to forget the trailing `!` on macro invocations, which would also\nyield this error:\n\n```compile_fail,E0423\nprintln(\"\");\n// error: expected function, tuple struct or tuple variant,\n// found macro `println`\n// did you mean `println!(...)`? (notice the trailing `!`)\n```\n\nAnother case where this error is emitted is when a value is expected, but\nsomething else is found:\n\n```compile_fail,E0423\npub mod a {\n    pub const I: i32 = 1;\n}\n\nfn h1() -> i32 {\n    a.I\n    //~^ ERROR expected value, found module `a`\n    // did you mean `a::I`?\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpec6fuvso","byte_start":597,"byte_end":608,"line_start":30,"line_end":30,"column_start":27,"column_end":38,"is_primary":true,"text":[{"text":"        MyOption::None => unreachable(),","highlight_start":27,"highlight_end":38}],"label":"not a function","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"use `!` to invoke the macro","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpec6fuvso","byte_start":608,"byte_end":608,"line_start":30,"line_end":30,"column_start":38,"column_end":38,"is_primary":true,"text":[{"text":"        MyOption::None => unreachable(),","highlight_start":38,"highlight_end":38}],"label":null,"suggested_replacement":"!","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null},{"message":"consider importing this function instead","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpec6fuvso","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use std::intrinsics::unreachable;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0423]: expected function, found macro `unreachable`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpec6fuvso:30:27\n   |\n30 |         MyOption::None => unreachable(),\n   |                           ^^^^^^^^^^^ not a function\n   |\nhelp: use `!` to invoke the macro\n   |\n30 |         MyOption::None => unreachable!(),\n   |                                      +\nhelp: consider importing this function instead\n   |\n1  + use std::intrinsics::unreachable;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0423`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0423`.\n"}
// 
// 