use vstd::prelude::*;
use vstd::atomic::*;
use vstd::invariant::*;
use vstd::cell;
use vstd::cell::*;
use vstd::atomic;
use vstd::modes::*;

verus!{

struct LockInv { }

impl<T> InvariantPredicate<(AtomicCellId, CellId), (atomic::PermissionBool, Option<cell::PointsTo<T>>)> for LockInv {
    open spec fn inv(
        cell_ids: (AtomicCellId, CellId),
        ghost_stuff: (atomic::PermissionBool, Option<cell::PointsTo<T>>),
    ) -> bool {
        true
    }
}

struct Lock<T> {
    pub atomic: PAtomicBool,
    pub cell: PCell<T>,
    pub inv: Tracked<AtomicInvariant<
        (AtomicCellId, CellId),
        (atomic::PermissionBool, Option<cell::PointsTo<T>>),
        LockInv
    >>,
}

impl<T> Lock<T> {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.inv.constant() == (self.atomic.id(), self.cell.id())
        &&& self.inv.namespace() == 1337
        &&& is_sized::<T>()
    }

    pub fn new(t: T) -> (lock: Self)
        ensures
            lock.view() == (false, t),
    {
        let (atomic, Tracked(atomic_perm)) = PAtomicBool::new(false);
        let (cell, Tracked(cell_perm)) = PCell::new(t);
        let tracked inv = AtomicInvariant::new(
            (atomic.id(), cell.id()),
            (atomic_perm, Some(cell_perm)),
            1337);
        Lock { atomic, cell, inv: Tracked(inv) }
    }

    pub fn acquire(&self) -> (points_to: Tracked<cell::PointsTo<T>>)
        ensures
            self.view() == old(self).view(),
            points_to@.value() == self.view().1,
        opens_invariants 1337
    {
        proof { use_type_invariant(&*self); }
        loop
            invariant true,
        {
            let tracked points_to_opt = None;
            let res;
            open_atomic_invariant!(self.inv.borrow() => ghost_stuff => {
                let tracked (mut atomic_permission, mut points_to_inv) = ghost_stuff;
                res = self.atomic.compare_exchange(Tracked(&mut atomic_permission), false, true);
                proof {
                    tracked_swap(&mut points_to_opt, &mut points_to_inv);
                    ghost_stuff = (atomic_permission, points_to_inv);
                }
            });
            if res.is_ok() {
                return Tracked(points_to_opt.tracked_unwrap());
            }
        }
    }

    pub fn release(&self, points_to: Tracked<cell::PointsTo<T>>)
        requires
            points_to@.pcell == self.cell.id(),
        ensures
            self.view() == old(self).view(),
        opens_invariants 1337
    {
        proof { use_type_invariant(&*self); }
        open_atomic_invariant!(self.inv.borrow() => ghost_stuff => {
            let tracked (mut atomic_permission, _) = ghost_stuff;
            self.atomic.store(Tracked(&mut atomic_permission), false);
            proof {
                ghost_stuff = (atomic_permission, Some(points_to.get()));
            }
        });
    }
}

impl<T> View for Lock<T> {
    type V = (bool, T);

    closed spec fn view(&self) -> Self::V {
        (false, self.cell@.value())
    }
}

pub fn test_lock_generic()
    ensures
        true,
{
    let lock = Lock::new(42);
    let points_to = lock.acquire();
    lock.release(points_to);
}

}

fn main() { }

// Step 4 (spec_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// Verified: -1, Errors: 999, Verus Errors: 1