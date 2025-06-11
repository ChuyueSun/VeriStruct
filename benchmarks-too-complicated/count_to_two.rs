#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use vstd::prelude::*;

use std::sync::Arc;
use vstd::atomic::*;
use vstd::atomic_ghost::*;
use vstd::invariant::*;

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
        !(self is Invalid)
    }

    open spec fn op(self, other: Self) -> Self {
        match (self, other) {
            (OneShotResourceValue::Empty, _) => other,
            (_, OneShotResourceValue::Empty) => self,
            (
                OneShotResourceValue::HalfRightToComplete,
                OneShotResourceValue::HalfRightToComplete,
            ) => OneShotResourceValue::FullRightToComplete {  },
            (
                OneShotResourceValue::Complete,
                OneShotResourceValue::Complete,
            ) => OneShotResourceValue::Complete {  },
            (_, _) => OneShotResourceValue::Invalid {  },
        }
    }

    open spec fn unit() -> Self {
        OneShotResourceValue::Empty {  }
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
        self.r.loc()
    }

    pub closed spec fn view(self) -> OneShotResourceValue {
        self.r.value()
    }

    // This function creates a one-shot and returns a resource
    // granting the full authority to perform the created
    // one-shot.
    pub proof fn alloc() -> (tracked resource: Self)
        ensures
            resource@ is FullRightToComplete,
    {
        let v = OneShotResourceValue::FullRightToComplete {  };
        let tracked mut r = Resource::<OneShotResourceValue>::alloc(v);
        OneShotResource { r }
    }

    // This function splits full authority to perform a one-shot
    // into two half authorities to perform it.
    pub proof fn split(tracked self) -> (tracked return_value: (Self, Self))
        requires
            self@ is FullRightToComplete,
        ensures
            ({
                let (half1, half2) = return_value;
                &&& half1@ is HalfRightToComplete
                &&& half2@ is HalfRightToComplete
                &&& half2.id() == half1.id() == self.id()
            }),
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
            old(self)@ is FullRightToComplete,
        ensures
            self@ is Complete,
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
        requires
            old(other).id() == old(self).id(),
            old(self)@ is HalfRightToComplete,
            !(old(other)@ is Empty),
        ensures
            old(other)@ is HalfRightToComplete,
            self@ is Complete,
            other@ is Complete,
            other.id() == self.id() == old(self).id(),
    {
        self.r.validate();
        other.r.validate();
        // A `HalfRightToComplete` doesn't combine validly with a
        // `Complete`, so validating them together proves that
        // `other.r.value()` is `HalfRightToComplete`.
        self.r.validate_2(&other.r);
        assert(other@ is HalfRightToComplete);
        // Knowing they're both `HalfRightToComplete` allows them to
        // be combined and transformed into `Complete` resources.
        let v = OneShotResourceValue::Complete {  };
        update_and_redistribute(&mut self.r, &mut other.r, v, v);
    }

    // This function duplicates a one-shot resource representing
    // knowledge of completion.
    pub proof fn duplicate(tracked &self) -> (tracked other: Self)
        requires
            self@ is Complete,
        ensures
            other.id() == self.id(),
            other@ is Complete,
    {
        let tracked r = duplicate(&self.r);
        Self { r }
    }

    pub proof fn lemma_is_complete_if_other_is(tracked &mut self, tracked other: &Self)
        requires
            other.id() == old(self).id(),
            other@ is Complete,
            !(old(self)@ is Empty),
        ensures
            self.id() == old(self).id(),
            self@ == old(self)@,
            self@ is Complete,
    {
        self.r.validate_2(&other.r);
    }
}


// This struct holds all the ghost tracked state that the counter
// will keep in an invariant.
//
// `x_perm` -- permission to write to the shared atomic variable `x`
//
// `oneshot0_inv_half` -- the invariant's resource for thread 0's
// one-shot, which contains either half the authority to complete that
// one-shot or knowledge that that one-shot has been performed
//
// `oneshot1_inv_half` -- as above, but for thread 1's one-shot
pub struct CounterTrackedState {
    pub x_perm: PermissionU32,
    pub oneshot0_inv_half: OneShotResource,
    pub oneshot1_inv_half: OneShotResource,
}

// This struct describes what's constant in the counter invariant.
//
// `x_id` -- the identity of the shared atomic variable `x`, which
// links the permission to write it with the actual atomic
// variable.
//
// `oneshot0_id` -- the ID of thread 0's one-shot
//
// `oneshot1_id` -- the ID of thread 1's one-shot
pub struct CounterInvariantConstants {
    pub x_id: int,
    pub oneshot0_id: int,
    pub oneshot1_id: int,
}

// This is the invariant predicate that will be maintained for the
// `CounterTrackedState`.
pub struct CounterInvariantPredicate {}

impl InvariantPredicate<
    CounterInvariantConstants,
    CounterTrackedState,
> for CounterInvariantPredicate {
    open spec fn inv(c: CounterInvariantConstants, cts: CounterTrackedState) -> bool {
        // The IDs of the resources held match those in the constants
        &&& cts.x_perm@.patomic == c.x_id
        &&& cts.oneshot0_inv_half.id() == c.oneshot0_id
        &&& cts.oneshot1_inv_half.id()
            == c.oneshot1_id
        // For each thread's one-shot, the invariant holds a resource that's either
        // (1) half authority to complete that one-shot or (2) knowledge that that
        // one-shot is complete.

        &&& cts.oneshot0_inv_half@ is HalfRightToComplete || cts.oneshot0_inv_half@ is Complete
        &&& cts.oneshot1_inv_half@ is HalfRightToComplete
            || cts.oneshot1_inv_half@ is Complete
        // The key invariant is that the value of `x` is the count
        // of how many threads' one-shots have completed.

        &&& cts.x_perm@.value == (if cts.oneshot0_inv_half@ is Complete {
            1int
        } else {
            0int
        }) + (if cts.oneshot1_inv_half@ is Complete {
            1int
        } else {
            0int
        })
    }
}

// This `CounterSharedState` struct is shared among the threads, using
// an atomic reference counter (Arc).
//
// `x` -- the actual counter implemented as an atomic u32
// `inv` -- the invariant holding the shared counter tracked state
pub struct CounterSharedState {
    pub x: PAtomicU32,
    pub inv: Tracked<
        AtomicInvariant<CounterInvariantConstants, CounterTrackedState, CounterInvariantPredicate>,
    >,
}

impl CounterSharedState {
    // This is the well-formedness predicate for a `CounterSharedState`.
    pub open spec fn wf(self) -> bool {
        &&& self.x.id() == self.inv@.constant().x_id
        &&& self.inv@.namespace() == 888
    }

    // This function gets, from the shared state's constants, the ID
    // of the one-shot associated with the given thread.
    pub open spec fn get_oneshot_id(self, which_thread: int) -> int
        recommends
            which_thread == 0 || which_thread == 1,
    {
        let c = self.inv@.constant();
        if which_thread == 0 {
            c.oneshot0_id
        } else {
            c.oneshot1_id
        }
    }

    // This function creates a new `CounterSharedState`.
    pub fn new(
        Tracked(oneshot0_inv_half): Tracked<OneShotResource>,
        Tracked(oneshot1_inv_half): Tracked<OneShotResource>,
    ) -> (result: Arc<Self>)
        requires
            oneshot0_inv_half@ is HalfRightToComplete,
            oneshot1_inv_half@ is HalfRightToComplete,
        ensures
            result.wf(),
            result.get_oneshot_id(0) == oneshot0_inv_half.id(),
            result.get_oneshot_id(1) == oneshot1_inv_half.id(),
    {
        // Create the atomic variable to be shared among threads.
        let (x, Tracked(x_perm)): (PAtomicU32, Tracked<PermissionU32>) = PAtomicU32::new(0);
        // Create the `CounterTrackedState`.
        let tracked cts = CounterTrackedState { x_perm, oneshot0_inv_half, oneshot1_inv_half };
        // Create the invariant.
        let ghost c = CounterInvariantConstants {
            x_id: x.id(),
            oneshot0_id: oneshot0_inv_half.id(),
            oneshot1_id: oneshot1_inv_half.id(),
        };
        assert(CounterInvariantPredicate::inv(c, cts));  // This is obvious, so no proof is needed.
        let inv = Tracked(AtomicInvariant::new(c, cts, 888));
        // Create the shared state to be shared among the threads
        // using Arcs.
        Arc::new(CounterSharedState { x, inv })
    }

    // This function reads the value of `x` from the `PAtomicU32`
    // that's part of this `CounterSharedState`. It requires, as
    // input, two `OneShotResource`s, one showing that thread 0's
    // one-shot is complete and the other showing that thread 1's
    // one-shot is complete. Because of these, it can ensure that the
    // value it reads is 2.
    pub fn read_x(
        self: &Arc<Self>,
        Tracked(oneshot0_complete): Tracked<OneShotResource>,
        Tracked(oneshot1_complete): Tracked<OneShotResource>,
    ) -> (x: u32)
        requires
            self.wf(),
            oneshot0_complete.id() == self.get_oneshot_id(0),
            oneshot1_complete.id() == self.get_oneshot_id(1),
            oneshot0_complete@ is Complete,
            oneshot1_complete@ is Complete,
        ensures  x == 2,
    {
        let x_value: u32;
        open_atomic_invariant!(self.inv.borrow() => inner => {
            proof {
                // Since `oneshot0_complete` reflects thread 0's
                // one-shot having completed, we can conclude that the
                // invariant's `oneshot0_inv_half` is also
                // `Completed`. After all, it's not possible for a
                // `HalfRightToComplete` and `Completed` resource to
                // co-exist for the same one-shot ID. We use
                // `lemma_is_complete_if_other_is` to show this.

                inner.oneshot0_inv_half.lemma_is_complete_if_other_is(&oneshot0_complete);

                // Similarly for `oneshot1_complete` and thread 1's
                // one-shot.

                inner.oneshot1_inv_half.lemma_is_complete_if_other_is(&oneshot1_complete);

                // The invariant says that the value of `x` is equal to
                //
                // ```
                // (if cts.oneshot0_inv_half@ is Complete { 1int } else { 0int }) +
                // (if cts.oneshot1_inv_half@ is Complete { 1int } else { 0int })
                // ```
                //
                // Since we know both have completed, we know `x == 2`. So,
                // when we load its value, that's what we'll get.
            }
            x_value = self.x.load(Tracked(&inner.x_perm));
            assert(x_value == 2); // This is the key assertion we needed to prove.
        });
        x_value
    }
}

// This is the routine that each thread will execute when forked. It
// increments the counter atomically with performing the one-shot.
//
// `shared_state` -- an Arc pointing to the state shared between
// threads
//
// `oneshot_thread_half` -- permission granting half of the
// authority to this thread's one-shot resource
//
// `which_thread` -- which thread this is, 0 or 1
pub fn thread_routine(
    shared_state: Arc<CounterSharedState>,
    Tracked(oneshot_thread_half): Tracked<OneShotResource>,
    Ghost(which_thread): Ghost<int>,
) -> (return_permission: Tracked<OneShotResource>)
    requires
        // which_thread == 0 || which_thread == 1,
        oneshot_thread_half@ is HalfRightToComplete,
        shared_state.wf(),
        oneshot_thread_half.id() == shared_state.get_oneshot_id(which_thread),
    ensures
        return_permission@.id() == shared_state.get_oneshot_id(which_thread),
        return_permission@@ is Complete,
{
    let tracked mut oneshot_thread_half = oneshot_thread_half;
    open_atomic_invariant!(shared_state.inv.borrow() => inner => {
        // Increment the shared `x` by 1.
        shared_state.x.fetch_add_wrapping(Tracked(&mut inner.x_perm), 1);
        proof {
            // Atomically with that increment, perform the one-shot.
            // This requires providing two half authorities. One was
            // passed to this function as `oneshot_thread_half` and
            // the other is in this invariant.
            //
            // Technically, the invariant just tells us that either
            // the one-shot is complete *or* we have half authority to
            // it. Fortunately, `perform_using_two_halves` only
            // requires that one of the resources be known to be a
            // half authority. (It can deduce that the other one must
            // be, since a `HalfRightToComplete` resource can't
            // co-exist with a `Completed` resource of the same ID.)
            if which_thread == 0 {
                oneshot_thread_half.perform_using_two_halves(&mut inner.oneshot0_inv_half);
            }
            else {
                oneshot_thread_half.perform_using_two_halves(&mut inner.oneshot1_inv_half);
            }
            assert(oneshot_thread_half@ is Complete);
        }
    });
    // Return the updated permission. It's been updated from (a)
    // half the authority to complete the one-shot to (b)
    // knowledge that the one-shot is complete.
    Tracked(oneshot_thread_half)
}

// This function counts to two by forking two threads, each tasked
// with incrementing `x` and then returning knowledge that that
// thread has performed its increment. In this way, it's able to
// guarantee that when it reads `x` after joining those two
// threads, the result is 2.
pub fn count_to_two() -> (result: Result<u32, ()>)
    ensures  result is Ok ==> result.unwrap() == 2,
{
    // Create two one-shots, one for each thread we're going to
    // fork. Calling `create_oneshot` provides two permissions to
    // the one-shot resource, each granting half authority to
    // perform the one-shot. We'll stash one half in our invariant
    // and pass the other half to the appropriate thread. It's
    // necessary to have both halves to perform any one-shot, so
    // each thread will have to combine its half with the
    // corresponding one in the invariant.
    let tracked (mut oneshot0_inv_half, mut oneshot0_thread_half) =
        OneShotResource::alloc().split();
    let tracked (mut oneshot1_inv_half, mut oneshot1_thread_half) =
        OneShotResource::alloc().split();
    // Create the shared state that includes a new `PAtomicU32` and
    // an invariant that starts out holding `oneshot0_inv_half` and
    // `oneshot1_inv_half1.
    let shared_state = CounterSharedState::new(
        Tracked(oneshot0_inv_half),
        Tracked(oneshot1_inv_half),
    );
    // For each thread, clone the shared-state Arc and use this to
    // fork the thread. Also pass each thread a tracked permission
    // providing half the authority to update its one-shot.
    let shared_state_clone = shared_state.clone();
    let join_handle0 = vstd::thread::spawn(
        move || -> (return_value: Tracked<OneShotResource>)
            ensures
                return_value@.id() == shared_state.get_oneshot_id(0),
                return_value@@ is Complete,
            { thread_routine(shared_state_clone, Tracked(oneshot0_thread_half), Ghost(0)) }
    );
    let shared_state_clone = shared_state.clone();
    let join_handle1 = vstd::thread::spawn(
        move || -> (return_value: Tracked<OneShotResource>)
            ensures
                return_value@.id() == shared_state.get_oneshot_id(1),
                return_value@@ is Complete,
            { thread_routine(shared_state_clone, Tracked(oneshot1_thread_half), Ghost(1)) }
    );
    // Let the threads run in parallel, then join them both when
    // they're done.
    if let (Ok(oneshot0_complete), Ok(oneshot1_complete)) = (
        join_handle0.join(),
        join_handle1.join(),
    ) {
        // If both joins succeeded, we can now read the shared
        // `PAtomicU32`'s value `x` by opening the invariant.
        Ok(shared_state.read_x(oneshot0_complete, oneshot1_complete))
    } else {
        // If either of the joins failed, we can't proceed.
        Err(())
    }
}

/* TEST CODE BELOW */

 fn main() {
    if let Ok(x) = count_to_two() {
        assert(x == 2);
    }
}
} // verus!
