use vstd::pervasive::*;
use builtin_macros::*;

verus! {

#[verifier::loop_isolation(false)]

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

fn main() {}


//             Option::Some(a) => (true, ( a ) as A),
//   the trait `builtin::Integer` is not implemented for `A`: A

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1