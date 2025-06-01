#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use std::result::*;
use vstd::pcm::*;
use vstd::pcm_lib::*;
use vstd::prelude::*;

verus! {

// A one-shot resource represents one of the following four resources:
//
// `FullRightToComplete` -- the authority to complete the one-shot;
//
// `HalfRightToComplete` -- half of the authority to complete the
// one-shot, which can be combined with another half to make a full
// authority; or
//
// `Complete` -- knowledge that the one-shot has completed.
//
// `Empty` - no permission at all.
pub enum OneShotResourceValue {
    FullRightToComplete,
    HalfRightToComplete,
    Complete,
    Empty,
    Invalid,
}

// To use `OneShotResourceValue` as a resource, we have to implement
// `PCM`, showing how to use it in a resource algebra.
impl PCM for OneShotResourceValue {
    open spec fn valid(self) -> bool {
        // TODO: implement specification.
        false
    }

    open spec fn op(self, other: Self) -> Self {
        // TODO: implement specification.
        OneShotResourceValue::Invalid
    }

    open spec fn unit() -> Self {
        // TODO: implement specification.
        OneShotResourceValue::Empty
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

pub struct OneShotResource {
    r: Resource<OneShotResourceValue>,
}

impl OneShotResource {
    pub closed spec fn id(self) -> Loc {
        // TODO: implement specification.
        zero_loc()
    }

    // REFINED View FUNCTION (flattened tuple)
    pub closed spec fn view(self) -> (Loc, bool, bool, bool, bool, bool) {
        let val = self.r@;
        (
            self.id(),
            val == OneShotResourceValue::FullRightToComplete,
            val == OneShotResourceValue::HalfRightToComplete,
            val == OneShotResourceValue::Complete,
            val == OneShotResourceValue::Empty,
            val == OneShotResourceValue::Invalid
        )
    }

    #[verifier::type_invariant]
    pub closed spec fn inv(&self) -> bool {
        self.r@.valid() && self.r.loc() == self.id()
    }

    // This function creates a one-shot and returns a resource
    // granting the full authority to perform the created one-shot.
    pub proof fn alloc() -> (tracked resource: Self)
        ensures
            resource@.1,      // FullRightToComplete
            !resource@.2,     // not HalfRightToComplete
            !resource@.3,     // not Complete
            !resource@.4,     // not Empty
            !resource@.5,     // not Invalid
    {
        let v = OneShotResourceValue::FullRightToComplete {  };
        let tracked mut r = Resource::<OneShotResourceValue>::alloc(v);
        OneShotResource { r }
    }

    // This function splits full authority to perform a one-shot
    // into two half authorities to perform it.
    pub proof fn split(tracked self) -> (tracked return_value: (Self, Self))
        requires
            self@.1, // must be FullRightToComplete
        ensures
            return_value.0@.2 && return_value.1@.2, // both are HalfRightToComplete
            !return_value.0@.1 && !return_value.1@.1,
            !return_value.0@.3 && !return_value.1@.3,
            !return_value.0@.4 && !return_value.1@.4,
            !return_value.0@.5 && !return_value.1@.5,
            return_value.0@.0 == return_value.1@.0 && return_value.0@.0 == self@.0,
    {
        let half = OneShotResourceValue::HalfRightToComplete {  };
        let tracked (r1, r2) = self.r.split(half, half);
        (OneShotResource { r: r1 }, OneShotResource { r: r2 })
    }

    // This function performs a one-shot given a resource representing
    // full authority to complete the one-shot.
    //
    // Upon return, the passed-in resource will have been transformed
    // into knowledge that the one-shot has been performed.
    pub proof fn perform(tracked &mut self)
        requires
            self@.1, // must be FullRightToComplete
        ensures
            self@.3,   // now Complete
            !self@.1,  // no longer FullRightToComplete
            !self@.2,  // not HalfRightToComplete
            !self@.4,  // not Empty
            !self@.5,  // not Invalid
            self@.0 == old(self)@.0,
    {
        let v = OneShotResourceValue::Complete {  };
        update_mut(&mut self.r, v);
    }

    // This function performs a one-shot given two resources, the
    // first of which represents an incomplete one-shot (and half the
    // authority needed to perform it). The resources must have the
    // same `id()`, meaning they're talking about the same one-shot.
    //
    // Upon return, the passed-in resources will have both been
    // transformed into knowledge that the one-shot has been
    // performed.
    pub proof fn perform_using_two_halves(tracked &mut self, tracked other: &mut Self)
        requires
            self@.2,         // self must be HalfRightToComplete
            !other@.4,       // other must not be Empty
            self@.0 == other@.0,
        ensures
            self@.3 && other@.3,    // both become Complete
            !self@.1 && !other@.1,  // no longer FullRightToComplete
            !self@.2 && !other@.2,  // no longer HalfRightToComplete
            !self@.4 && !other@.4,  // not Empty
            !self@.5 && !other@.5,  // not Invalid
            self@.0 == old(self)@.0,
            other@.0 == old(other)@.0,
    {
        self.r.validate();
        other.r.validate();
        self.r.validate_2(&other.r);
        assert(other@ is HalfRightToComplete);
        let v = OneShotResourceValue::Complete {  };
        update_and_redistribute(&mut self.r, &mut other.r, v, v);
    }

    // This function duplicates a one-shot resource representing
    // knowledge of completion.
    pub proof fn duplicate(tracked &self) -> (tracked other: Self)
        requires
            self@.3,  // must already be Complete
        ensures
            other@.3,
            !other@.1,
            !other@.2,
            !other@.4,
            !other@.5,
            other@.0 == self@.0,
    {
        let tracked r = duplicate(&self.r);
        Self { r }
    }

    pub proof fn lemma_is_complete_if_other_is(tracked &mut self, tracked other: &Self)
        requires
            self@.0 == other@.0,
        ensures
            self@.0 == old(self)@.0,
            if other@.3 { self@.3 },
    {
        self.r.validate_2(&other.r);
    }
}

// This example illustrates some uses of the one-shot functions.
fn main() {
    let tracked full = OneShotResource::alloc();
    proof {
        full.perform();
    }
    assert(full@ is Complete);
    let tracked different_oneshot = OneShotResource::alloc();
    let tracked (mut half1, mut half2) = different_oneshot.split();
    let ghost id = half1.id();
    assert(half1.id() == half2.id());
    assert(half1@ is HalfRightToComplete);
    assert(half2@ is HalfRightToComplete);
    proof {
        half1.perform_using_two_halves(&mut half2);
    }
    assert(half1.id() == half2.id() == id);
    assert(half1@ is Complete);
    assert(half2@ is Complete);
    let tracked knowledge = half1.duplicate();
    assert(knowledge.id() == half1.id() == id);
    assert(knowledge@ is Complete);
}

} // verus!

// Step 4 (spec_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1