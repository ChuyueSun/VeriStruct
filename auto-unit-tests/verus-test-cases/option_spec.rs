use vstd::pervasive::*;
use builtin_macros::*;
use vstd::prelude::*;

verus! {

#[verifier::ext_equal]
#[verifier::accept_recursive_types(A)]
pub enum MyOption<A> {
    None,
    Some(A),
}

pub open spec fn is_Some<A>(opt: MyOption<A>) -> bool {
    matches!(opt, MyOption::Some(_))
}

pub open spec fn is_None<A>(opt: MyOption<A>) -> bool {
    matches!(opt, MyOption::None)
}

pub open spec fn get_Some_0<A>(opt: MyOption<A>) -> A
{
    match opt {
        MyOption::Some(a) => a,
        MyOption::None => arbitrary(),
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
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    pub fn or(self, optb: MyOption<A>) -> (res: MyOption<A>)
    ensures
        res == self.Or(optb)
    {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> (res: bool)
        ensures
            res <==> is_Some(*self)
    {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> (res: bool)
        ensures
            res <==> is_None(*self),
    {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> (a: MyOption<&A>)
        ensures
            is_Some(a) <==> is_Some(*self),
            is_Some(a) ==> get_Some_0(*self) == get_Some_0(a),
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
}

/* TEST CODE BELOW */