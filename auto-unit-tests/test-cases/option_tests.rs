use std::clone::Clone;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_some() {
        let some: MyOption<i32> = MyOption::Some(10);
        let none: MyOption<i32> = MyOption::None;
        assert!(some.is_some());
        assert!(!none.is_some());
    }

    #[test]
    fn test_is_none() {
        let some: MyOption<&str> = MyOption::Some("hello");
        let none: MyOption<&str> = MyOption::None;
        assert!(!some.is_none());
        assert!(none.is_none());
    }

    #[test]
    fn test_or_with_some_self() {
        let first: MyOption<i32> = MyOption::Some(1);
        let second: MyOption<i32> = MyOption::Some(2);
        // When self is Some, it should return self regardless of the second option.
        match first.or(second.clone()) {
            MyOption::Some(val) => assert_eq!(val, 1),
            MyOption::None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_or_with_none_self() {
        let first: MyOption<i32> = MyOption::None;
        let second: MyOption<i32> = MyOption::Some(2);
        // When self is None, it should return the second option.
        match first.or(second.clone()) {
            MyOption::Some(val) => assert_eq!(val, 2),
            MyOption::None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_or_both_none() {
        let first: MyOption<i32> = MyOption::None;
        let second: MyOption<i32> = MyOption::None;
        // When both options are None, the result should be None.
        assert!(matches!(first.or(second), MyOption::None));
    }

    #[test]
    fn test_as_ref_some() {
        let value = 100;
        let some: MyOption<i32> = MyOption::Some(value);
        match some.as_ref() {
            MyOption::Some(&val) => assert_eq!(val, value),
            MyOption::None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_as_ref_none() {
        let none: MyOption<i32> = MyOption::None;
        assert!(matches!(none.as_ref(), MyOption::None));
    }

    #[test]
    fn test_unwrap_some() {
        let some: MyOption<&str> = MyOption::Some("test");
        assert_eq!(some.unwrap(), "test");
    }

    #[test]
    #[should_panic]
    fn test_unwrap_none() {
        let none: MyOption<i32> = MyOption::None;
        // This should panic because unwrap is called on a None variant.
        none.unwrap();
    }

    #[test]
    fn test_clone_some() {
        let original: MyOption<String> = MyOption::Some(String::from("clone me"));
        let cloned = original.clone();
        match cloned {
            MyOption::Some(val) => assert_eq!(val, "clone me"),
            MyOption::None => panic!("Expected Some variant after cloning"),
        }
    }

    #[test]
    fn test_clone_none() {
        let original: MyOption<String> = MyOption::None;
        let cloned = original.clone();
        assert!(matches!(cloned, MyOption::None));
    }
}