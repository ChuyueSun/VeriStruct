pub enum MyOption<A> {
    None,
    Some(A),
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
    pub fn or(self, optb: MyOption<A>) -> MyOption<A> {
        match self {
            MyOption::None => optb,
            MyOption::Some(_) => self,
        }
    }

    #[inline(always)]
    pub const fn is_some(&self) -> bool {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    #[inline(always)]
    pub const fn is_none(&self) -> bool {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }

    pub fn as_ref(&self) -> MyOption<&A> {
        match self {
            MyOption::Some(x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> A {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => unreachable!(),
        }
    }
}

/* TEST CODE BELOW */

fn test(n: i32) {
    let opt: MyOption<i32> = MyOption::None;
    let is_none = opt.is_none();
    let is_some = opt.is_some();
    assert!(is_none);
    assert!(!is_some);

    let opt2: MyOption<i32> = MyOption::Some(n);
    let is_none = opt2.is_none();
    let is_some = opt2.is_some();
    assert!(!is_none);
    assert!(is_some);

    let opt3 = opt.or(opt2);
    let is_some = opt3.is_some();
    let val = opt3.unwrap();
    assert!(is_some);
    assert!(val == n);

    let opt4 = opt2.or(opt);
    let is_some = opt4.is_some();
    let val = opt4.unwrap();
    assert!(is_some);
    assert!(val == n);

    let opt5 = opt.or(MyOption::None);
    let is_none = opt5.is_none();
    let is_some = opt5.is_some();
    assert!(is_none);
    assert!(!is_some);

    let opt_some: MyOption<i32> = MyOption::Some(n);
    let opt_ref = opt_some.as_ref();
    let ref_some = opt_ref.is_some();
    let val = *opt_ref.unwrap();
    assert!(ref_some);
    assert!(val == n);

    let opt_none: MyOption<i32> = MyOption::None;
    let opt_ref_none = opt_none.as_ref();
    let ref_none = opt_none.is_none();
    assert!(ref_none);
}

pub fn main() {
}