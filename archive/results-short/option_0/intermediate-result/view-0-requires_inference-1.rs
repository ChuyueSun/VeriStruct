use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

//----------------------------------------------
// Spec functions (no requires/ensures allowed)
//----------------------------------------------

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    match opt {
        MyOption::Some(_) => true,
        MyOption::None => false,
    }
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool {
    !is_Some(opt)
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A;

// Spec definition of "Or"
pub open spec fn Or<A>(opt1: MyOption<A>, opt2: MyOption<A>) -> MyOption<A> {
    if is_Some(opt1) { opt1 } else { opt2 }
}

//----------------------------------------------
// Trait implementations
//----------------------------------------------

impl<A: Clone> Clone for MyOption<A> {
    fn clone(&self) -> Self {
        match self {
            MyOption::None => MyOption::None,
            MyOption::Some(a) => MyOption::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for MyOption<A> {}

//----------------------------------------------
// MyOption methods
//----------------------------------------------

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A> {
        Or(self, optb)
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        ensures
            if self.is_some() {
                res == self
            } else {
                res == optb
            }
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
            if self.is_some() {
                a.is_some()
            } else {
                a.is_none()
            },
            if self.is_some() {
                get_Some_0(a) == &get_Some_0(*self)
            }
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

//----------------------------------------------
// Test code (no changes)
//----------------------------------------------

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
// VerusErrorType.Other: `if` may be missing an `else` clause
// {"$message_type":"diagnostic","message":"`if` may be missing an `else` clause","code":{"code":"E0317","explanation":"An `if` expression is missing an `else` block.\n\nErroneous code example:\n\n```compile_fail,E0317\nlet x = 5;\nlet a = if x == 5 {\n    1\n};\n```\n\nThis error occurs when an `if` expression without an `else` block is used in a\ncontext where a type other than `()` is expected. In the previous code example,\nthe `let` expression was expecting a value but since there was no `else`, no\nvalue was returned.\n\nAn `if` expression without an `else` block has the type `()`, so this is a type\nerror. To resolve it, add an `else` block having the same type as the `if`\nblock.\n\nSo to fix the previous code example:\n\n```\nlet x = 5;\nlet a = if x == 5 {\n    1\n} else {\n    2\n};\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpjvgb1u31","byte_start":2362,"byte_end":2447,"line_start":102,"line_end":104,"column_start":13,"column_end":14,"is_primary":true,"text":[{"text":"            if self.is_some() {","highlight_start":13,"highlight_end":32},{"text":"                get_Some_0(a) == &get_Some_0(*self)","highlight_start":1,"highlight_end":52},{"text":"            }","highlight_start":1,"highlight_end":14}],"label":"expected `bool`, found `()`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpjvgb1u31","byte_start":2398,"byte_end":2433,"line_start":103,"line_end":103,"column_start":17,"column_end":52,"is_primary":false,"text":[{"text":"                get_Some_0(a) == &get_Some_0(*self)","highlight_start":17,"highlight_end":52}],"label":"found here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"`if` expressions without `else` evaluate to `()`","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider adding an `else` block that evaluates to the expected type","code":null,"level":"help","spans":[],"children":[],"rendered":null}],"rendered":"error[E0317]: `if` may be missing an `else` clause\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpjvgb1u31:102:13\n    |\n102 | /             if self.is_some() {\n103 | |                 get_Some_0(a) == &get_Some_0(*self)\n    | |                 ----------------------------------- found here\n104 | |             }\n    | |_____________^ expected `bool`, found `()`\n    |\n    = note: `if` expressions without `else` evaluate to `()`\n    = help: consider adding an `else` block that evaluates to the expected type\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0317`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0317`.\n"}
//
//
