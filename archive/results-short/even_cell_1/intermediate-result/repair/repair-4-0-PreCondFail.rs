use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

#[verifier::loop_isolation(false)]
ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        points_to.is_init()
        && points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>) -> (res: ())
    requires
        true,
    ensures
        true,
{
    open_local_invariant!(inv => points_to => {
        assert(points_to.is_init());
        assert(points_to.value() % 2 == 0);

        let x = cell.take(Tracked(&mut points_to));
        assert(x % 2 == 0);

        proof {
            assert(!points_to.is_init());
        }

        let x_plus_2 = if x == 254 { 0 } else { x + 2 };
        cell.put(Tracked(&mut points_to), x_plus_2);

        assert(points_to.is_init());
        assert(points_to.value() % 2 == 0);
    });
}

fn main() {
    let (cell, Tracked(points_to)) = PCell::new(4);

    let tracked inv = LocalInvariant::new(
        cell.id(),
        points_to,
        1337);

    add_2(&cell, Tracked(&inv));
    add_2(&cell, Tracked(&inv));
    add_2(&cell, Tracked(&inv));
}
}

//         cell.put(Tracked(&mut points_to), x_plus_2);
// failed precondition
//             old(perm)@ === pcell_opt![ self.id() => Option::None ],
//   failed precondition: old(perm)@ === pcell_opt![ self.id() => Option::None ]
//   None: cell.put(Tracked(&mut points_to), x_plus_2)

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
