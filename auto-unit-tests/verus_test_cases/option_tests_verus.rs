use vstd::prelude::*;

verus! {

    pub enum MyOption<A> {
        None,
        Some(A),
    }

    impl<A: Clone> Clone for MyOption<A> {
        fn clone(&self) -> Self
            ensures
                match self {
                    MyOption::None => result == MyOption::None,
                    MyOption::Some(a) => result == MyOption::Some(a.clone()),
                }
        {
            match self {
                MyOption::None => MyOption::None,
                MyOption::Some(a) => MyOption::Some(a.clone()),
            }
        }
    }

    impl<A: Copy> Copy for MyOption<A> {}

    impl<A> MyOption<A> {
        pub fn or(self, optb: MyOption<A>) -> MyOption<A>
            ensures
                match self {
                    MyOption::Some(_) => result == self,
                    MyOption::None => result == optb,
                }
        {
            match self {
                MyOption::None => optb,
                MyOption::Some(_) => self,
            }
        }

        #[inline(always)]
        pub const fn is_some(&self) -> bool
            ensures
                result == (match self {
                    MyOption::Some(_) => true,
                    MyOption::None => false,
                })
        {
            match self {
                MyOption::Some(_) => true,
                MyOption::None => false,
            }
        }

        #[inline(always)]
        pub const fn is_none(&self) -> bool
            ensures
                result == (match self {
                    MyOption::Some(_) => false,
                    MyOption::None => true,
                })
        {
            match self {
                MyOption::Some(_) => false,
                MyOption::None => true,
            }
        }

        pub fn as_ref(&self) -> MyOption<&A>
            ensures
                match self {
                    MyOption::Some(x) => result == MyOption::Some(x),
                    MyOption::None => result == MyOption::None,
                }
        {
            match self {
                MyOption::Some(x) => MyOption::Some(x),
                MyOption::None => MyOption::None,
            }
        }

        pub fn unwrap(self) -> A
            requires
                self.is_some(),
            ensures
                match self {
                    MyOption::Some(a) => result == a,
                    MyOption::None => false, // unreachable because of the precondition
                }
        {
            match self {
                MyOption::Some(a) => a,
                MyOption::None => unreachable!(),
            }
        }
    }

    /* TEST CODE BELOW */

    pub fn main() {
        // Test: test_is_some_and_is_none_on_some
        let opt = MyOption::Some(10);
        assert(opt.is_some());
        assert(!opt.is_none());

        // Test: test_is_some_and_is_none_on_none
        let opt_none: MyOption<i32> = MyOption::None;
        assert(!opt_none.is_some());
        assert(opt_none.is_none());

        // Test: test_or_with_some_and_none
        let some_val = MyOption::Some(5);
        let none_val: MyOption<i32> = MyOption::None;
        // When self is Some, or should return self.
        let res = some_val.or(none_val);
        assert(res.is_some());
        // When self is None, or should return optb.
        let res2 = none_val.or(some_val);
        assert(res2.is_some());

        // Test: test_or_both_none
        let none1: MyOption<i32> = MyOption::None;
        let none2: MyOption<i32> = MyOption::None;
        let res3 = none1.or(none2);
        assert(res3.is_none());

        // Test: test_as_ref_on_some
        let opt2 = MyOption::Some(42);
        let ref_opt = opt2.as_ref();
        match ref_opt {
            MyOption::Some(val) => { assert(*val == 42); }
            MyOption::None => { assert(false); }
        }

        // Test: test_as_ref_on_none
        let opt3: MyOption<i32> = MyOption::None;
        let ref_opt2 = opt3.as_ref();
        assert(ref_opt2.is_none());

        // Test: test_unwrap_on_some
        let opt4 = MyOption::Some("Hello");
        let val = opt4.unwrap();
        assert(val == "Hello");

        // Test: test_clone
        let opt5 = MyOption::Some(100);
        let clone_opt = opt5.clone();
        match clone_opt {
            MyOption::Some(val) => { assert(val == 100); }
            MyOption::None => { assert(false); }
        }

        // Test: test_copy_trait
        let opt6 = MyOption::Some(200);
        let copied = opt6; // Copy occurs here.
        match copied {
            MyOption::Some(val) => { assert(val == 200); }
            MyOption::None => { assert(false); }
        }
        assert(opt6.is_some());
    }
}