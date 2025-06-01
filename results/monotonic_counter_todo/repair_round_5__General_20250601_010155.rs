#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

// A monotonic counter permission represents a resource with one of
// the following four values:
//
// `LowerBound { lower_bound }`    -- knowledge that the monotonic counter
// is at least `lower_bound`
//
// `HalfRightToAdvance { value }`  -- knowledge that the monotonic counter is
// exactly `value` and half the authority to advance it
//
// `FullRightToAdvance { value }`  -- knowledge that the monotonic counter is
// exactly `value` and the full authority to advance it
//
// `Invalid` -- special variant indicating no valid permission
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
        /* TODO: part of view */
        match self {
            MonotonicCounterResourceValue::Invalid => false,
            _ => true,
        }
    }

    open spec fn op(self, other: Self) -> Self {
        /* TODO: part of view */
        MonotonicCounterResourceValue::Invalid
    }

    open spec fn unit() -> Self {
        /* TODO: part of view */
        MonotonicCounterResourceValue::LowerBound { lower_bound: 0 }
    }

    proof fn closed_under_incl(a: Self, b: Self) {
        /* TODO: part of view */
    }

    proof fn commutative(a: Self, b: Self) {
        /* TODO: part of view */
    }

    proof fn associative(a: Self, b: Self, c: Self) {
        /* TODO: part of view */
    }

    proof fn op_unit(a: Self) {
        /* TODO: part of view */
    }

    proof fn unit_valid() {
        /* TODO: part of view */
    }
}

impl MonotonicCounterResourceValue {
    pub open spec fn n(self) -> nat {
        /* TODO: part of view */
        match self {
            MonotonicCounterResourceValue::LowerBound { lower_bound } => lower_bound,
            MonotonicCounterResourceValue::HalfRightToAdvance { value } => value,
            MonotonicCounterResourceValue::FullRightToAdvance { value } => value,
            MonotonicCounterResourceValue::Invalid => 0,
        }
    }
}

pub struct MonotonicCounterResource {
    r: Resource<MonotonicCounterResourceValue>,
}

impl MonotonicCounterResource {
    pub closed spec fn id(self) -> Loc {
        /* TODO: part of view */
        self.r.loc()
    }

    pub closed spec fn view(self) -> MonotonicCounterResourceValue {
        /* TODO: part of view */
        self.r@
    }

    // This function creates a monotonic counter and returns a
    // resource granting full authority to advance it and giving
    // knowledge that the current value is 0.
    pub proof fn alloc() -> (tracked result: Self)
    // TODO: add requires and ensures
    {
        let v = MonotonicCounterResourceValue::FullRightToAdvance { value: 0 };
        let tracked mut r = Resource::<MonotonicCounterResourceValue>::alloc(v);
        Self { r }
    }

    // This function splits a resource granting full authority to
    // advance a monotonic counter into two resources each granting
    // half authority to advance it. They both have the same `id()`,
    // meaning they correspond to the same monotonic counter.
    pub proof fn split(tracked self) -> (tracked return_value: (Self, Self))
    // TODO: add requires and ensures
    {
        let value = self@.n();
        // matches self@ must be FullRightToAdvance to even do this
        // for brevity, we extract the 'value' field directly
        let v_half = MonotonicCounterResourceValue::HalfRightToAdvance { value };
        let tracked (r1, r2) = self.r.split(v_half, v_half);
        (Self { r: r1 }, Self { r: r2 })
    }

    // This function uses a resource granting full authority to
    // advance a monotonic counter to increment the counter.
    pub proof fn increment(tracked &mut self)
    // TODO: add requires and ensures
    {
        let v = self@.n();
        let r = MonotonicCounterResourceValue::FullRightToAdvance { value: v + 1 };
        update_mut(&mut self.r, r);
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
        self.r.validate_2(&other.r);
        let v = self@.n();
        let r = MonotonicCounterResourceValue::HalfRightToAdvance { value: v + 1 };
        update_and_redistribute(&mut self.r, &mut other.r, r, r);
    }

    // Extract a lower-bound resource from the current resource
    pub proof fn extract_lower_bound(tracked &self) -> (tracked out: Self)
    // TODO: add requires and ensures
    {
        self.r.validate();
        let v = MonotonicCounterResourceValue::LowerBound { lower_bound: self@.n() };
        let tracked r = copy_duplicable_part(&self.r, v);
        Self { r }
    }
}

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
}

} // verus!

// Repair Round 5 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1