use vstd::prelude::*;
use vstd::atomic_ghost::*;
use vstd::cell;
use vstd::cell::*;
use vstd::modes::*;

verus!{

struct_with_invariants!{
    struct Lock<T> {
        pub atomic: AtomicBool<_, Option<cell::PointsTo<T>>, _>,
        pub cell: PCell<T>,
    }

    spec fn wf(self) -> bool {
        // TODO: add specification
    }
}

impl<T> Lock<T> {
    fn new(t: T) -> (lock: Self)
        // TODO: add requires and ensures
    {
        let (cell, Tracked(cell_perm)) = PCell::new(t);
        let atomic = AtomicBool::new(Ghost(cell), false, Tracked(Some(cell_perm)));
        Lock { atomic, cell }
    }

    fn acquire(&self) -> (points_to: Tracked<cell::PointsTo<T>>)
        // TODO: add requires and ensures
    {
        loop
            invariant self.wf(),
        {
            let tracked mut points_to_opt = None;
            let res = atomic_with_ghost!(&self.atomic => compare_exchange(false, true);
                ghost points_to_inv => {
                    tracked_swap(&mut points_to_opt, &mut points_to_inv);
                }
            );
            if res.is_ok() {
                return Tracked(points_to_opt.tracked_unwrap());
            }
        }
    }

    fn release(&self, points_to: Tracked<cell::PointsTo<T>>)
        // TODO: add requires and ensures
    {
        atomic_with_ghost!(&self.atomic => store(false);
            ghost points_to_inv => {
                points_to_inv = Some(points_to.get());
            }
        );
    }
}
fn test_lock_generic() {
    let lock = Lock::new(42);
    let points_to = lock.acquire();
    lock.release(points_to);
}
}

fn main() { }
