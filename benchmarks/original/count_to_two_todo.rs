#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use vstd::prelude::*;

pub mod oneshot;
use crate::oneshot::*;
use std::sync::Arc;
use vstd::atomic::*;
use vstd::atomic_ghost::*;
use vstd::invariant::*;

verus! {

pub struct CounterTrackedState {
    pub x_perm: PermissionU32,
    pub oneshot0_inv_half: OneShotResource,
    pub oneshot1_inv_half: OneShotResource,
}
pub struct CounterInvariantConstants {
    pub x_id: int,
    pub oneshot0_id: int,
    pub oneshot1_id: int,
}
pub struct CounterInvariantPredicate {}

impl InvariantPredicate<
    CounterInvariantConstants,
    CounterTrackedState,
> for CounterInvariantPredicate {
    open spec fn inv(c: CounterInvariantConstants, cts: CounterTrackedState) -> bool {
        // TODO: add specification
    }
}

pub struct CounterSharedState {
    pub x: PAtomicU32,
    pub inv: Tracked<
        AtomicInvariant<CounterInvariantConstants, CounterTrackedState, CounterInvariantPredicate>,
    >,
}

impl CounterSharedState {
    pub open spec fn wf(self) -> bool {
        // TODO: add specification
    }

    pub open spec fn get_oneshot_id(self, which_thread: int) -> int
    {
        // TODO: add specification
    }

    pub fn new(
        Tracked(oneshot0_inv_half): Tracked<OneShotResource>,
        Tracked(oneshot1_inv_half): Tracked<OneShotResource>,
    ) -> (result: Arc<Self>)
    // TODO: add requires and ensures
    {
        let (x, Tracked(x_perm)): (PAtomicU32, Tracked<PermissionU32>) = PAtomicU32::new(0);
        let tracked cts = CounterTrackedState { x_perm, oneshot0_inv_half, oneshot1_inv_half };
        let ghost c = CounterInvariantConstants {
            x_id: x.id(),
            oneshot0_id: oneshot0_inv_half.id(),
            oneshot1_id: oneshot1_inv_half.id(),
        };
        assert(CounterInvariantPredicate::inv(c, cts));
        let inv = Tracked(AtomicInvariant::new(c, cts, 888));
        Arc::new(CounterSharedState { x, inv })
    }
    pub fn read_x(
        self: &Arc<Self>,
        Tracked(oneshot0_complete): Tracked<OneShotResource>,
        Tracked(oneshot1_complete): Tracked<OneShotResource>,
    ) -> (x: u32)
    // TODO: add requires and ensures
    {
        let x_value: u32;
        open_atomic_invariant!(self.inv.borrow() => inner => {
            proof {

                inner.oneshot0_inv_half.lemma_is_complete_if_other_is(&oneshot0_complete);

                inner.oneshot1_inv_half.lemma_is_complete_if_other_is(&oneshot1_complete);

            }
            x_value = self.x.load(Tracked(&inner.x_perm));
            assert(x_value == 2);
        });
        x_value
    }
}

pub fn thread_routine(
    shared_state: Arc<CounterSharedState>,
    Tracked(oneshot_thread_half): Tracked<OneShotResource>,
    Ghost(which_thread): Ghost<int>,
) -> (return_permission: Tracked<OneShotResource>)
// TODO: add requires and ensures
{
    let tracked mut oneshot_thread_half = oneshot_thread_half;
    open_atomic_invariant!(shared_state.inv.borrow() => inner => {
        shared_state.x.fetch_add_wrapping(Tracked(&mut inner.x_perm), 1);
        proof {
            if which_thread == 0 {
                oneshot_thread_half.perform_using_two_halves(&mut inner.oneshot0_inv_half);
            }
            else {
                oneshot_thread_half.perform_using_two_halves(&mut inner.oneshot1_inv_half);
            }
            assert(oneshot_thread_half@ is Complete);
        }
    });
    Tracked(oneshot_thread_half)
}

pub fn count_to_two() -> (result: Result<u32, ()>)
// TODO: add requires and ensures
{
    let tracked (mut oneshot0_inv_half, mut oneshot0_thread_half) =
        OneShotResource::alloc().split();
    let tracked (mut oneshot1_inv_half, mut oneshot1_thread_half) =
        OneShotResource::alloc().split();
    let shared_state = CounterSharedState::new(
        Tracked(oneshot0_inv_half),
        Tracked(oneshot1_inv_half),
    );
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
    if let (Ok(oneshot0_complete), Ok(oneshot1_complete)) = (
        join_handle0.join(),
        join_handle1.join(),
    ) {
        Ok(shared_state.read_x(oneshot0_complete, oneshot1_complete))
    } else {
        Err(())
    }
}

pub fn main() {
    if let Ok(x) = count_to_two() {
        assert(x == 2);
    }
}

} // verus!
