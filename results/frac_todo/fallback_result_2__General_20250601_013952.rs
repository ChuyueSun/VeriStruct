use builtin::*;
use vstd::pcm::*;
use vstd::prelude::*;

// This implements a resource for fractional ownership of a ghost variable.
// The fractions are represented as some number out of a compile-time const
// Total value; you can have any fractions from 1 up to Total, and if you
// have Total, you can update the ghost variable.

verus! {
    pub enum Fractional<T, const Total: u64> {
        Value { v: T, n: int },
        Empty,
        Invalid,
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // View Abstraction for Fractional<T, Total>
    ////////////////////////////////////////////////////////////////////////////////////

    /// A spec-level enum to represent the Fractional variants at the specification level.
    pub spec enum FractionalViewOf<T> {
        Value { v: T, n: int },
        Empty,
        Invalid,
    }

    impl<T, const Total: u64> View for Fractional<T, Total> {
        type V = FractionalViewOf<T>;

        closed spec fn view(&self) -> Self::V {
            match self {
                Fractional::Value { v, n } => FractionalViewOf::Value { v, n },
                Fractional::Empty => FractionalViewOf::Empty,
                Fractional::Invalid => FractionalViewOf::Invalid,
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Fractional PCM Implementation
    ////////////////////////////////////////////////////////////////////////////////////

    impl<T, const Total: u64> Fractional<T, Total> {
        pub open spec fn new(v: T) -> Self {
            /* TODO: implement specification. */
            Fractional::Empty // placeholder
        }
    }

    impl<T, const Total: u64> PCM for Fractional<T, Total> {
        open spec fn valid(self) -> bool {
            /* TODO: implement specification. */
            true
        }

        open spec fn op(self, other: Self) -> Self {
            /* TODO: implement specification. */
            Fractional::Empty
        }

        open spec fn unit() -> Self {
            /* TODO: implement specification. */
            Fractional::Empty
        }

        proof fn closed_under_incl(a: Self, b: Self) {
            // no-op
        }

        proof fn commutative(a: Self, b: Self) {
            // no-op
        }

        proof fn associative(a: Self, b: Self, c: Self) {
            // no-op
        }

        proof fn op_unit(a: Self) {
            // no-op
        }

        proof fn unit_valid() {
            // no-op
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // FractionalResource
    ////////////////////////////////////////////////////////////////////////////////////

    pub struct FractionalResource<T, const Total: u64> {
        r: Resource<Fractional<T, Total>>,
    }

    impl<T, const Total: u64> FractionalResource<T, Total> {
        pub closed spec fn inv(self) -> bool
        {
            // TODO: implement specification.
            true
        }

        pub closed spec fn id(self) -> Loc
        {
            // TODO: implement specification.
            arbitrary()
        }

        pub closed spec fn val(self) -> T
        {
            // TODO: implement specification.
            arbitrary()
        }

        pub closed spec fn frac(self) -> int
        {
            // TODO: implement specification.
            0
        }

        pub proof fn alloc(v: T) -> (tracked result: FractionalResource<T, Total>)
            // TODO: add ensures and requires
        {
            let f = Fractional::<T, Total>::new(v);
            let tracked r = Resource::alloc(f);
            FractionalResource { r }
        }

        pub proof fn agree(tracked self: &mut FractionalResource<T, Total>, tracked other: &FractionalResource<T, Total>)
           // TODO: add ensures and requires
        {
            self.r.validate_2(&other.r)
        }

        pub proof fn split(tracked self, n: int) ->
            (tracked result: (FractionalResource<T, Total>, FractionalResource<T, Total>))
            // TODO: add ensures and requires
        {
            let tracked (r1, r2) = self.r.split(
                Fractional::Value { v: self.r.value().v, n: self.r.value().n - n },
                Fractional::Value { v: self.r.value().v, n: n }
            );
            (FractionalResource { r: r1 }, FractionalResource { r: r2 })
        }

        pub proof fn combine(tracked self, tracked other: FractionalResource<T, Total>) -> (tracked result: FractionalResource<T, Total>)
            // TODO: add ensures and requires
        {
            let tracked mut mself = self;
            mself.r.validate_2(&other.r);
            let tracked r = mself.r.join(other.r);
            FractionalResource { r: r }
        }

        pub proof fn update(tracked self, v: T) -> (tracked result: FractionalResource<T, Total>)
            // TODO: add ensures and requires
        {
            let f = Fractional::<T, Total>::Value { v: v, n: Total as int };
            let tracked r = self.r.update(f);
            FractionalResource { r: r }
        }
    }

    fn main()
    {
        let tracked r = FractionalResource::<u64, 3>::alloc(123);
        assert(r.val() == 123);
        assert(r.frac() == 3);
        let tracked (r1, r2) = r.split(2);
        assert(r1.val() == 123);
        assert(r2.val() == 123);
        assert(r1.frac() == 1);
        assert(r2.frac() == 2);
        let tracked r3 = r1.combine(r2);
        let tracked r4 = r3.update(456);
        assert(r4.val() == 456);
        assert(r4.frac() == 3);
        ()
    }
}

// Fallback VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1