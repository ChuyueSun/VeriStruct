use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        points_to.id() == cell_id
        && points_to.is_init()
        && points_to.value() % 2 == 0
    }
}

/* 
TEST CODE BELOW
*/

fn test_even_cell(
    cell: &PCell<u8>, 
    Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>, 
    v: u8
) requires
    inv.constant() == cell.id(),
    v % 2 == 0
{
    open_local_invariant!(inv => points_to => {
        assert(points_to.is_init());
        assert(points_to.value() % 2 == 0);

        let x = cell.take(Tracked(&mut points_to));
        assert(x % 2 == 0);

        cell.put(Tracked(&mut points_to), v);

        assert(points_to.is_init());
        assert(points_to.value() % 2 == 0);
    });
}

fn main() {
    let (cell, Tracked(points_to)) = PCell::new(4);

    let tracked inv = LocalInvariant::new(
        cell.id(),
        points_to,
        1337 /* arbitrary namespace */);

    test_even_cell(&cell, Tracked(&inv), 18);
    test_even_cell(&cell, Tracked(&inv), 30);
    test_even_cell(&cell, Tracked(&inv), 130);
}

}
