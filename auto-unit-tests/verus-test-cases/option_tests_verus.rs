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
pub fn main() {
    // Test is_some and is_none on Some and None
    let some_val: MyOption<u32> = MyOption::Some(10);
    let none_val: MyOption<u32> = MyOption::None;

    assert(some_val.is_some());
    assert(!some_val.is_none());
    assert(!none_val.is_some());
    assert(none_val.is_none());

    // Test unwrap on Some, ensuring it returns the expected value.
    let unwrapped_val = some_val.unwrap();
    assert(unwrapped_val == 10);

    // Test or: When self is Some, 'or' should return self.
    let res1 = some_val.or(none_val);
    assert(is_Some(res1));
    assert(res1.unwrap() == 10);

    // Test or: When self is None, 'or' should return optb.
    let res2 = none_val.or(some_val);
    assert(is_Some(res2));
    assert(res2.unwrap() == 10);

    // Test as_ref on Some: the underlying value should be preserved.
    let ref_some = some_val.as_ref();
    assert(is_Some(ref_some));
    assert(get_Some_0(some_val) == get_Some_0(ref_some));

    // Test as_ref on None: should return None.
    let ref_none = none_val.as_ref();
    assert(is_None(ref_none));
}
}