use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool
    ensures
        result == (opt is MyOption::Some(_)),
{
    // TODO: add specification
    unimplemented!()
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool
    ensures
        result == (opt is MyOption::None),
{
    // TODO: add specification
    unimplemented!()
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> (res: A)
    requires
        is_Some(opt),
    ensures
        MyOption::Some(res) == opt,
{
    // TODO: add specification
    unimplemented!()
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
    pub open spec fn Or(self, optb: MyOption<A>) -> MyOption<A>
        ensures
            result == if is_Some(self) { self } else { optb },
    {
        // TODO: add specification
        unimplemented!()
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
        ensures
            res == if is_Some(self) { self } else { optb },
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
            (is_Some(*self) <==> is_Some(a)),
            forall|x: A| self == MyOption::Some(x) ==> a == MyOption::Some(&x),
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
            MyOption::Some(a) == self,
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
            MyOption::Some(a) == self,
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
// VerusErrorType.Other: expected `,`
// {"$message_type":"diagnostic","message":"expected `,`","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpp9hrpayf","byte_start":269,"byte_end":270,"line_start":15,"line_end":15,"column_start":35,"column_end":36,"is_primary":true,"text":[{"text":"        result == (opt is MyOption::Some(_)),","highlight_start":35,"highlight_end":36}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected `,`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpp9hrpayf:15:35\n   |\n15 |         result == (opt is MyOption::Some(_)),\n   |                                   ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
