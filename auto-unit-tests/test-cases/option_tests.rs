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
        let some_value = MyOption::Some(5);
        let none_value: MyOption<i32> = MyOption::None;
        assert!(some_value.is_some());
        assert!(!none_value.is_some());
    }

    #[test]
    fn test_is_none() {
        let some_value = MyOption::Some(10);
        let none_value: MyOption<i32> = MyOption::None;
        assert!(!some_value.is_none());
        assert!(none_value.is_none());
    }

    #[test]
    fn test_or_method() {
        // Case: Some.or(None) should yield Some
        let option1 = MyOption::Some(1);
        let option2: MyOption<i32> = MyOption::None;
        let result = option1.or(option2);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 1);

        // Case: None.or(Some) should yield Some
        let option1: MyOption<i32> = MyOption::None;
        let option2 = MyOption::Some(2);
        let result = option1.or(option2);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 2);

        // Case: None.or(None) should yield None
        let option1: MyOption<i32> = MyOption::None;
        let option2: MyOption<i32> = MyOption::None;
        let result = option1.or(option2);
        assert!(result.is_none());

        // Case: Some.or(Some) should retain the first Some
        let option1 = MyOption::Some(3);
        let option2 = MyOption::Some(4);
        let result = option1.or(option2);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_as_ref() {
        let some_value = MyOption::Some(20);
        match some_value.as_ref() {
            MyOption::Some(&val) => assert_eq!(val, 20),
            MyOption::None => panic!("Expected Some, got None"),
        }

        let none_value: MyOption<i32> = MyOption::None;
        match none_value.as_ref() {
            MyOption::Some(_) => panic!("Expected None, got Some"),
            MyOption::None => {}
        }
    }

    #[test]
    fn test_unwrap_some() {
        let some_value = MyOption::Some("hello");
        assert_eq!(some_value.unwrap(), "hello");
    }

    #[test]
    #[should_panic]
    fn test_unwrap_none() {
        let none_value: MyOption<i32> = MyOption::None;
        let _ = none_value.unwrap();
    }

    #[test]
    fn test_clone_and_copy() {
        // Using a type that supports Clone and Copy (i32)
        let original = MyOption::Some(42);
        let cloned = original.clone();
        assert_eq!(original.unwrap(), cloned.unwrap());

        // Test Copy trait: copying should work implicitly
        let copied = original;
        assert_eq!(copied.unwrap(), 42);
    }
}