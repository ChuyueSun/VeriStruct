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
    fn test_is_some_and_is_none_on_some() {
        let opt = MyOption::Some(10);
        assert!(opt.is_some());
        assert!(!opt.is_none());
    }

    #[test]
    fn test_is_some_and_is_none_on_none() {
        let opt: MyOption<i32> = MyOption::None;
        assert!(!opt.is_some());
        assert!(opt.is_none());
    }

    #[test]
    fn test_or_with_some_and_none() {
        let some_val = MyOption::Some(5);
        let none_val: MyOption<i32> = MyOption::None;
        // If self is Some, or should return self
        let res = some_val.or(none_val);
        assert!(res.is_some());
        // If self is None, or should return optb
        let res2 = none_val.or(some_val);
        assert!(res2.is_some());
    }

    #[test]
    fn test_or_both_none() {
        let none1: MyOption<i32> = MyOption::None;
        let none2: MyOption<i32> = MyOption::None;
        let res = none1.or(none2);
        assert!(res.is_none());
    }

    #[test]
    fn test_as_ref_on_some() {
        let opt = MyOption::Some(42);
        let ref_opt = opt.as_ref();
        // Check that ref_opt is Some and dereferences to the same value
        match ref_opt {
            MyOption::Some(val) => assert_eq!(*val, 42),
            MyOption::None => panic!("Expected Some value"),
        }
    }

    #[test]
    fn test_as_ref_on_none() {
        let opt: MyOption<i32> = MyOption::None;
        let ref_opt = opt.as_ref();
        assert!(ref_opt.is_none());
    }

    #[test]
    fn test_unwrap_on_some() {
        let opt = MyOption::Some("Hello");
        let val = opt.unwrap();
        assert_eq!(val, "Hello");
    }

    #[test]
    #[should_panic]
    fn test_unwrap_on_none_panics() {
        let opt: MyOption<i32> = MyOption::None;
        // This should panic
        let _ = opt.unwrap();
    }

    #[test]
    fn test_clone() {
        let opt = MyOption::Some(100);
        let clone = opt.clone();
        match clone {
            MyOption::Some(val) => assert_eq!(val, 100),
            MyOption::None => panic!("Clone of Some should be Some"),
        }
    }

    #[test]
    fn test_copy_trait() {
        // Only works for types implementing Copy.
        let opt = MyOption::Some(200);
        let copied = opt; // Copy should occur here.
        // Both opt and copied are valid because i32 is Copy.
        match copied {
            MyOption::Some(val) => assert_eq!(val, 200),
            MyOption::None => panic!("Copied value should be Some"),
        }
        // Also verify that the original remains intact.
        assert!(opt.is_some());
    }
}