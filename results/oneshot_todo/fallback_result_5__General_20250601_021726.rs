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

    pub closed spec fn view(self) -> OneShotResourceValue {
        // Filled in for the "TODO: part of view"
        self.r@
    }

    // This function creates a one-shot and returns a resource
    // granting the full authority to perform the created
    // one-shot.
    pub proof fn alloc() -> (tracked resource: Self)
    // TODO: add requires and ensures
    {
        let v = OneShotResourceValue::FullRightToComplete {  };
        let tracked mut r = Resource::<OneShotResourceValue>::alloc(v);
        OneShotResource { r }
    }

    // This function splits full authority to perform a one-shot
    // into two half authorities to perform it.
    pub proof fn split(tracked self) -> (tracked return_value: (Self, Self))
    // TODO: add requires and ensures
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
    // TODO: add requires and ensures
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
    //
    // The caller of this function only needs to know that `self`
    // provides half authority and that `other` isn't `Empty`. Upon
    // return the caller will learn that *both* the resources had
    // provided half authority at call time. However, those resources
    // were transformed so they don't provide that authority anymore.
    pub proof fn perform_using_two_halves(tracked &mut self, tracked other: &mut Self)
    // TODO: add requires and ensures
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
    // TODO: add requires and ensures
    {
        let tracked r = duplicate(&self.r);
        Self { r }
    }

    pub proof fn lemma_is_complete_if_other_is(tracked &mut self, tracked other: &Self)
    // TODO: add requires and ensures
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

// Fallback VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1