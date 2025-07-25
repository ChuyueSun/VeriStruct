use std::result::Result;

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

impl<A: Copy> Copy for MyOption<A> {}

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
        match *self {
            MyOption::Some(ref x) => MyOption::Some(x),
            MyOption::None => MyOption::None,
        }
    }

    pub fn unwrap(self) -> A {
        match self {
            MyOption::Some(a) => a,
            MyOption::None => panic!("called `MyOption::unwrap()` on a `None` value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_some_and_is_none() {
        let some_val: MyOption<i32> = MyOption::Some(10);
        assert!(some_val.is_some());
        assert!(!some_val.is_none());

        let none_val: MyOption<i32> = MyOption::None;
        assert!(none_val.is_none());
        assert!(!none_val.is_some());
    }

    #[test]
    fn test_or_method() {
        // When self is Some, the result should be self.
        let opt_a = MyOption::Some(5);
        let opt_b = MyOption::Some(10);
        let result = opt_a.or(opt_b);
        match result {
            MyOption::Some(val) => assert_eq!(val, 5),
            MyOption::None => panic!("or() should have returned Some"),
        }

        // When self is None, the result should be the provided optb.
        let opt_a: MyOption<i32> = MyOption::None;
        let opt_b = MyOption::Some(15);
        let result = opt_a.or(opt_b);
        match result {
            MyOption::Some(val) => assert_eq!(val, 15),
            MyOption::None => panic!("or() should have returned Some from second argument"),
        }

        // When both are None, the result should be None.
        let opt_a: MyOption<i32> = MyOption::None;
        let opt_b: MyOption<i32> = MyOption::None;
        let result = opt_a.or(opt_b);
        match result {
            MyOption::None => {},
            MyOption::Some(_) => panic!("or() should have returned None when both options are None"),
        }
    }

    #[test]
    fn test_as_ref() {
        let value = 42;
        let opt = MyOption::Some(value);
        let opt_ref = opt.as_ref();
        match opt_ref {
            MyOption::Some(&val) => assert_eq!(val, value),
            MyOption::None => panic!("as_ref() should return Some reference when option is Some"),
        }

        let none_opt: MyOption<i32> = MyOption::None;
        let none_ref = none_opt.as_ref();
        match none_ref {
            MyOption::None => {},
            MyOption::Some(_) => panic!("as_ref() should return None when option is None"),
        }
    }

    #[test]
    fn test_unwrap_success() {
        let opt = MyOption::Some(100);
        let val = opt.unwrap();
        assert_eq!(val, 100);
    }

    #[test]
    #[should_panic(expected = "called `MyOption::unwrap()` on a `None` value")]
    fn test_unwrap_failure() {
        let opt: MyOption<i32> = MyOption::None;
        // This should panic with the expected message.
        opt.unwrap();
    }

    #[test]
    fn test_clone_and_copy() {
        let opt = MyOption::Some(7);
        let cloned_opt = opt.clone();
        match cloned_opt {
            MyOption::Some(val) => assert_eq!(val, 7),
            MyOption::None => panic!("Clone did not produce a Some value"),
        }

        // Since i32 is Copy, the original option should be usable after copy.
        let copied_opt = opt;
        match copied_opt {
            MyOption::Some(val) => assert_eq!(val, 7),
            MyOption::None => panic!("Copy did not produce a Some value"),
        }
    }
}