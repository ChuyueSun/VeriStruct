//! This file implements monotonic counters using a custom resource
//! algebra.
//!
//! To use it, use MonotonicCounterResource::alloc(), which will
//! create a fresh monotonic counter and return a resource granting
//! full access to it. You can increment it the counter by calling
//! `increment` on a resource. For example:
//!
//! ```
//! let tracked full = MonotonicCounterResource::alloc();
//! proof { full.increment(); }
//! assert(full@.n() == 1);
//! ```
//!
//! To split a full right to advance into two half rights to advance,
//! use `split`. This is useful, for instance, to stash half inside an
//! invariant and pass the other half to the thread having the right
//! to advance. Both halves will have the same `id()` value,
//! indicating they correspond to the same monotonic counter. For
//! example:
//!
//! ```
//! let tracked full = MonotonicCounterResource::alloc();
//! let tracked (mut half1, mut half2) = full.split();
//! assert(half1.id() == half2.id());
//! assert(half1@.n() == 0);
//! assert(half2@.n() == 0);
//! ```
//!
//! You can use two half authorities together to increment the
//! associated counter, as in this example:
//!
//! ```
//! let ghost v1 == half1@.n();
//! proof { half1.increment_using_two_halves(&mut half2); }
//! assert(half1.id() == half2.id());
//! assert(half1@ == half2@);
//! assert(half1@.n() == half2@.n() == v1 + 1);
//! ```
//!
//! From any `MonotonicCounterResource`, one can use
//! `extract_lower_bound()` to extract a `MonotonicCounterResource`
//! that represents knowledge of a lower bound on the current value of
//! the monotonic counter. You can also duplicate a
//! `MonotonicCounterResource` using this function. Here are examples:
//!
//! ```
//! let tracked mut lower_bound = half1.extract_lower_bound();
//! assert(lower_bound@.n() == 1);
//! let tracked lower_bound_duplicate = lower_bound.extract_lower_bound();
//! assert(lower_bound_duplicate@.n() == 1);
//! ```
#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

// A monotonic counter permission represents a resource with one of
// the following three values:
//
// `LowerBound{ lower_bound }` -- knowledge that the monotonic counter
// is at least `lower_bound`
//
// `FullRightToAdvance{ value }` -- knowledge that the monotonic counter is
// exactly `value` and the authority to advance it past that value
//
// `HalfRightToAdvance{ value }` -- knowledge that the monotonic
// counter is exactly `value` and half the authority to advance it
// past that value. Can be combined with another half authority to
// make a full authority.
pub enum MonotonicCounterResourceValue {
    LowerBound { lower_bound: nat },
    HalfRightToAdvance { value: nat },
    FullRightToAdvance { value: nat },
    Invalid,
}

// To use `MonotonicCounterResourceValue` as a resource, we have to implement
// `PCM`, showing how to use it in a resource algebra.
impl PCM for MonotonicCounterResourceValue {
    open spec fn valid(self) -> bool {
        !(self is Invalid)
    }

    // Two lower bounds can be combined into a lower bound
    // that's the maximum of the two lower bounds.
    // A lower bound can be combined with a right to
    // advance as long as the lower bound doesn't exceed
    // the value in the right to advance.
    // A lower bound can be combined with a half right to
    // advance as long as the lower bound doesn't exceed
    // the value in the half right to advance.
    // Two half rights to advance can be combined to make
    // a whole right to advance, as long as the two values
    // agree with each other.
    // Any other combination is invalid
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

impl MonotonicCounterResourceValue {
    pub open spec fn n(self) -> nat {
        // TODO: add specification function
    }
}

pub struct MonotonicCounterResource {
    r: Resource<MonotonicCounterResourceValue>,
}

impl MonotonicCounterResource {
    pub closed spec fn id(self) -> Loc {
        // TODO: add specification function
    }

    pub closed spec fn view(self) -> MonotonicCounterResourceValue {
        // TODO: add specification function
    }

    // This function creates a monotonic counter and returns a
    // resource granting full authority to advance it and giving
    // knowledge that the current value is 0.
    pub proof fn alloc() -> (tracked result: Self)
    // TODO: add requires and ensures
    {
        // TODO: implement proof code
    }

    // This function splits a resource granting full authority to
    // advance a monotonic counter into two resources each granting
    // half authority to advance it. They both have the same `id()`,
    // meaning they correspond to the same monotonic counter.
    pub proof fn split(tracked self) -> (tracked return_value: (Self, Self))
    // TODO: add requires and ensures
    {
        // TODO: implement proof code
    }

    // This function uses a resource granting full authority to
    // advance a monotonic counter to increment the counter.
    pub proof fn increment(tracked &mut self)
    // TODO: add requires and ensures
    {
        // TODO: implement proof code
    }

    // This function uses two tracked resources, each granting half
    // authority to advance a monotonic counter, to increment the
    // counter. The two permissions must have the same `id()` values.
    //
    // It's not a requirement that the two halves match in value; this
    // function can figure out that they match just from the fact that
    // they co-exist.
    pub proof fn increment_using_two_halves(tracked &mut self, tracked other: &mut Self)
    // TODO: add requires and ensures
    {
        // TODO: implement proof code
    }

    pub proof fn extract_lower_bound(tracked &self) -> (tracked out: Self)
    // TODO: add requires and ensures
    {
        // TODO: implement proof code
    }
}

/* TEST CODE BELOW */

// This example illustrates some uses of the monotonic counter.
fn main() {
    let tracked full = MonotonicCounterResource::alloc();
    proof {
        full.increment();
    }
    assert(full@.n() == 1);
    let tracked full = MonotonicCounterResource::alloc();
    let tracked (mut half1, mut half2) = full.split();
    assert(half1.id() == half2.id());
    assert(half1@.n() == 0);
    assert(half2@.n() == 0);
    let ghost id = half1.id();
    let ghost v1 = half1@.n();
    let ghost v2 = half2@.n();
    assert(v1 == v2);
    proof {
        half1.increment_using_two_halves(&mut half2);
    }
    assert(half1.id() == half2.id() == id);
    assert(half1@.n() == half2@.n() == v1 + 1);
    assert(half1@.n() == 1);
    let tracked mut lower_bound = half1.extract_lower_bound();
    assert(lower_bound@.n() == 1);
    let tracked lower_bound_duplicate = lower_bound.extract_lower_bound();
    assert(lower_bound_duplicate@.n() == 1);

    // Test combining two lower bounds
    proof {
        let lb1 = MonotonicCounterResourceValue::LowerBound { lower_bound: 2 };
        let lb2 = MonotonicCounterResourceValue::LowerBound { lower_bound: 5 };
        let combined = lb1.op(lb2);
        assert(combined == MonotonicCounterResourceValue::LowerBound { lower_bound: 5 });
    }

    // Test combining lower bound and full right to advance (valid)
    proof {
        let lb = MonotonicCounterResourceValue::LowerBound { lower_bound: 3 };
        let full = MonotonicCounterResourceValue::FullRightToAdvance { value: 5 };
        let combined = lb.op(full);
        assert(combined == MonotonicCounterResourceValue::FullRightToAdvance { value: 5 });
    }

    // Test combining lower bound and full right to advance (invalid)
    proof {
        let lb = MonotonicCounterResourceValue::LowerBound { lower_bound: 7 };
        let full = MonotonicCounterResourceValue::FullRightToAdvance { value: 5 };
        let combined = lb.op(full);
        assert(combined == MonotonicCounterResourceValue::Invalid);
    }

    // Test combining two half rights to advance (valid)
    proof {
        let half1 = MonotonicCounterResourceValue::HalfRightToAdvance { value: 4 };
        let half2 = MonotonicCounterResourceValue::HalfRightToAdvance { value: 4 };
        let combined = half1.op(half2);
        assert(combined == MonotonicCounterResourceValue::FullRightToAdvance { value: 4 });
    }

    // Test combining two half rights to advance (invalid)
    proof {
        let half1 = MonotonicCounterResourceValue::HalfRightToAdvance { value: 4 };
        let half2 = MonotonicCounterResourceValue::HalfRightToAdvance { value: 5 };
        let combined = half1.op(half2);
        assert(combined == MonotonicCounterResourceValue::Invalid);
    }

    // Test combining lower bound and half right to advance (valid)
    proof {
        let lb = MonotonicCounterResourceValue::LowerBound { lower_bound: 2 };
        let half = MonotonicCounterResourceValue::HalfRightToAdvance { value: 3 };
        let combined = lb.op(half);
        assert(combined == MonotonicCounterResourceValue::HalfRightToAdvance { value: 3 });
    }

    // Test combining lower bound and half right to advance (invalid)
    proof {
        let lb = MonotonicCounterResourceValue::LowerBound { lower_bound: 5 };
        let half = MonotonicCounterResourceValue::HalfRightToAdvance { value: 3 };
        let combined = lb.op(half);
        assert(combined == MonotonicCounterResourceValue::Invalid);
    }

    // Test combining full right to advance and half right to advance (should be invalid)
    proof {
        let full = MonotonicCounterResourceValue::FullRightToAdvance { value: 2 };
        let half = MonotonicCounterResourceValue::HalfRightToAdvance { value: 2 };
        let combined = full.op(half);
        assert(combined == MonotonicCounterResourceValue::Invalid);
    }

    // Test unit element
    proof {
        let unit = MonotonicCounterResourceValue::unit();
        let lb = MonotonicCounterResourceValue::LowerBound { lower_bound: 3 };
        let combined = unit.op(lb);
        assert(combined == MonotonicCounterResourceValue::LowerBound { lower_bound: 3 });
        let combined2 = lb.op(unit);
        assert(combined2 == MonotonicCounterResourceValue::LowerBound { lower_bound: 3 });
    }
}

} // verus!
