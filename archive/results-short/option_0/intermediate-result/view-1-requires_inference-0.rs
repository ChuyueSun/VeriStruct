use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> (res: bool)
    ensures
        res == (opt != MyOption::None),
{
    arbitrary()
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> (res: bool)
    ensures
        res == (opt == MyOption::None),
{
    arbitrary()
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> (res: A)
    requires
        is_Some(opt),
    ensures
        MyOption::Some(res) == opt,
{
    arbitrary()
}

impl<A: Clone> Clone for MyOption<A> {
    fn clone(&self) -> Self {
        match self {
            MyOption::None => MyOption::None,
            MyOption::Some(a) => MyOption::Some(a.clone()),
        }
    }
}

impl<A: Copy> Copy for MyOption<A> { }

impl<A> MyOption<A> {
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A>
        ensures
            Or(self, optb) == (if self != MyOption::None { self } else { optb }),
    {
        arbitrary()
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        ensures
            res == (if self != MyOption::None { self } else { optb }),
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res == (self@ != MyOption::None),
    {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res == (self@ == MyOption::None),
    {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        ensures
            a == (if is_Some(self@) { MyOption::Some(get_Some_0(self@)) } else { MyOption::None }),
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
// VerusErrorType.Other: cannot find function, tuple struct or tuple variant `Or` in this scope
// {"$message_type":"diagnostic","message":"cannot find function, tuple struct or tuple variant `Or` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyyapl_or","byte_start":971,"byte_end":973,"line_start":50,"line_end":50,"column_start":13,"column_end":15,"is_primary":true,"text":[{"text":"            Or(self, optb) == (if self != MyOption::None { self } else { optb }),","highlight_start":13,"highlight_end":15}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider using the method on `Self`","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyyapl_or","byte_start":971,"byte_end":971,"line_start":50,"line_end":50,"column_start":13,"column_end":13,"is_primary":true,"text":[{"text":"            Or(self, optb) == (if self != MyOption::None { self } else { optb }),","highlight_start":13,"highlight_end":13}],"label":null,"suggested_replacement":"self.","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null},{"message":"try calling `Or` as a method","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyyapl_or","byte_start":971,"byte_end":985,"line_start":50,"line_end":50,"column_start":13,"column_end":27,"is_primary":true,"text":[{"text":"            Or(self, optb) == (if self != MyOption::None { self } else { optb }),","highlight_start":13,"highlight_end":27}],"label":null,"suggested_replacement":"self.Or(optb)","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0425]: cannot find function, tuple struct or tuple variant `Or` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyyapl_or:50:13\n   |\n50 | ...   Or(self, optb) == (if self != MyOption::None { self } else { optb }),\n   |       ^^\n   |\nhelp: consider using the method on `Self`\n   |\n50 |             self.Or(self, optb) == (if self != MyOption::None { self } else { optb }),\n   |             +++++\nhelp: try calling `Or` as a method\n   |\n50 |             self.Or(optb) == (if self != MyOption::None { self } else { optb }),\n   |             ~~~~~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0425`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0425`.\n"}
// 
// 