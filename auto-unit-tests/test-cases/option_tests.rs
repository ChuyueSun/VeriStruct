enum MyOption<A> {
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
            MyOption::None => panic!("called unwrap on None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_some_and_is_none() {
        let some_val: MyOption<i32> = MyOption::Some(10);
        let none_val: MyOption<i32> = MyOption::None;

        assert!(some_val.is_some());
        assert!(!some_val.is_none());

        assert!(none_val.is_none());
        assert!(!none_val.is_some());
    }

    #[test]
    fn test_or_method_when_self_is_some() {
        let opt1 = MyOption::Some(5);
        let opt2 = MyOption::Some(10);
        // when self is Some, or should return self regardless of opt2
        let result = opt1.or(opt2);
        match result {
            MyOption::Some(val) => assert_eq!(val, 5),
            MyOption::None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_or_method_when_self_is_none() {
        let opt1: MyOption<i32> = MyOption::None;
        let opt2 = MyOption::Some(20);
        let result = opt1.or(opt2);
        match result {
            MyOption::Some(val) => assert_eq!(val, 20),
            MyOption::None => panic!("Expected Some from second option, got None"),
        }
    }

    #[test]
    fn test_or_method_when_both_are_none() {
        let opt1: MyOption<i32> = MyOption::None;
        let opt2: MyOption<i32> = MyOption::None;
        let result = opt1.or(opt2);
        match result {
            MyOption::None => assert!(true),
            MyOption::Some(_) => panic!("Expected None, got Some"),
        }
    }

    #[test]
    fn test_as_ref_method() {
        let value = 30;
        let some_val = MyOption::Some(value);
        let none_val: MyOption<i32> = MyOption::None;

        let as_ref_some = some_val.as_ref();
        match as_ref_some {
            MyOption::Some(&v) => assert_eq!(v, 30),
            MyOption::None => panic!("Expected Some reference, got None"),
        }

        let as_ref_none = none_val.as_ref();
        match as_ref_none {
            MyOption::None => assert!(true),
            MyOption::Some(_) => panic!("Expected None, got Some"),
        }
    }

    #[test]
    fn test_unwrap_method_success() {
        let some_val = MyOption::Some("hello");
        let unwrapped = some_val.unwrap();
        assert_eq!(unwrapped, "hello");
    }

    #[test]
    #[should_panic(expected = "called unwrap on None")]
    fn test_unwrap_method_panic() {
        let none_val: MyOption<i32> = MyOption::None;
        let _ = none_val.unwrap();
    }

    #[test]
    fn test_clone_impl() {
        let original = MyOption::Some(String::from("rust"));
        let cloned = original.clone();
        match cloned {
            MyOption::Some(ref s) => assert_eq!(s, "rust"),
            MyOption::None => panic!("Expected Some, got None"),
        }

        let none_option: MyOption<String> = MyOption::None;
        let cloned_none = none_option.clone();
        match cloned_none {
            MyOption::None => assert!(true),
            MyOption::Some(_) => panic!("Expected None after cloning None"),
        }
    }

    #[test]
    fn test_copy_impl() {
        // Using an i32 which is Copy
        let original = MyOption::Some(100);
        let copied = original; // copy occurs here because i32 is Copy
        // Verify both original and copied still hold the value.
        match original {
            MyOption::Some(val) => assert_eq!(val, 100),
            MyOption::None => panic!("Expected Some, got None"),
        }
        match copied {
            MyOption::Some(val) => assert_eq!(val, 100),
            MyOption::None => panic!("Expected Some, got None"),
        }
    }
}