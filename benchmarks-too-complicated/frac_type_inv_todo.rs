use builtin::*;
use vstd::pcm::*;
use vstd::prelude::*;

// This implements a resource for fractional ownership of a ghost variable.
// The fractions are represented as some number out of a compile-time const
// Total value; you can have any fractions from 1 up to Total, and if you
// have Total, you can update the ghost variable.

verus! {

    // Too bad that `nat` and `int` are forbidden as the type of a const generic parameter
    pub enum Fractional<T, const Total: u64> {
        Value { v: T, n: int },
        Empty,
        Invalid,
    }

    impl<T, const Total: u64> Fractional<T, Total> {
        pub open spec fn new(v: T) -> Self {
            // TODO: add specification function
        }
    }

    impl<T, const Total: u64> PCM for Fractional<T, Total> {
        open spec fn valid(self) -> bool {
            // TODO: add specification function
        }

        open spec fn op(self, other: Self) -> Self {
            // TODO: add specification function
        }

        open spec fn unit() -> Self {
            // TODO: add specification function
        }

        proof fn closed_under_incl(a: Self, b: Self) {
        }

        proof fn commutative(a: Self, b: Self) {
        }

        proof fn associative(a: Self, b: Self, c: Self) {
        }

        proof fn op_unit(a: Self) {
        }

        proof fn unit_valid() {
        }
    }

    pub struct FractionalResource<T, const Total: u64> {
        r: Resource<Fractional<T, Total>>,
    }

    impl<T, const Total: u64> FractionalResource<T, Total> {
        #[verifier::type_invariant]
        pub closed spec fn inv(self) -> bool
        {
            // TODO: add specification function
        }

        pub closed spec fn id(self) -> Loc
        {
            // TODO: add specification function
        }

        pub closed spec fn val(self) -> T
        {
            // TODO: add specification function
        }

        pub closed spec fn frac(self) -> int
        {
            // TODO: add specification function
        }

        pub proof fn alloc(v: T) -> (tracked result: FractionalResource<T, Total>)
        // TODO: add requires and ensures
        {
            // TODO: add proof code
        }

        pub proof fn agree(tracked self: &mut FractionalResource<T, Total>, tracked other: &FractionalResource<T, Total>)
        // TODO: add requires and ensures
        {
            // TODO: add proof code
        }

        pub proof fn split(tracked self, n: int) ->
            (tracked result: (FractionalResource<T, Total>, FractionalResource<T, Total>))
        // TODO: add requires and ensures
        {
            // TODO: add proof code
        }

        pub proof fn combine(tracked self, tracked other: FractionalResource<T, Total>) -> (tracked result: FractionalResource<T, Total>)
        // TODO: add requires and ensures
        {
            // TODO: add proof code
        }

        pub proof fn update(tracked self, v: T) -> (tracked result: FractionalResource<T, Total>)
        // TODO: add requires and ensures
        {
            // TODO: add proof code
        }
    }

    /* TEST CODE BEGINS */

    fn test_fractional_resource_generic(v1: u64, v2: u64, m: int)
    requires
        0 < m < 100,
    {
        let tracked r = FractionalResource::<u64, 100>::alloc(v1);
        assert(r.val() == v1);
        assert(r.frac() == 100);
        let tracked (r1, r2) = r.split(m);
        assert(r1.val() == v1);
        assert(r2.val() == v1);
        assert(r1.frac() == 100 - m);
        assert(r2.frac() == m);
        let tracked r3 = r1.combine(r2);
        let tracked r4 = r3.update(v2);
        assert(r4.val() == v2);
        assert(r4.frac() == 100);
        ()
    }

    fn test_fractional_generic(v: u64)
    requires
        v >= 1,
    {
        proof {
            // the full fraction
            let f_full = Fractional::<u64, 100>::new(v);
            // empty element
            let f_empty = Fractional::<u64,100>::Empty;

            // identity laws
            assert(f_full.op(f_empty) == f_full);
            assert(f_empty.op(f_full) == f_full);

            // two valid pieces that sum to total
            let f1 = Fractional::<u64,100>::Value { v, n: 30 };
            let f2 = Fractional::<u64,100>::Value { v, n: 70 };
            let f_sum = f1.op(f2);
            // they combine to the full piece
            assert(f_sum == f_full);
            // commutativity
            assert(f2.op(f1) == f_full);

            // mismatched values → Invalid
            let f_badval = Fractional::<u64,100>::Value { v: (v - 1) as u64, n: 10 };
            assert(f1.op(f_badval) == Fractional::<u64,100>::Invalid);

            // non‐positive fraction → Invalid
            let f_zero = Fractional::<u64,100>::Value { v, n: 0 };
            assert(f1.op(f_zero) == Fractional::<u64,100>::Invalid);

            let f_exc = Fractional::<u64,100>::Value { v, n: 71 };
            assert(f1.op(f_zero) == Fractional::<u64,100>::Invalid);

        }
    }

    fn main() {}
}
