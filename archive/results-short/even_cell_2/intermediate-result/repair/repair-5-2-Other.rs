use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;
use vstd::cell::MemContents;

verus!{

#[verifier::loop_isolation(false)]

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        &&& points_to.pptr() == cell_id
        &&& points_to.mem_contents() matches MemContents::Init(val)
        &&& val % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>) -> ()
    requires
        cell.id() == inv.id(),
    ensures
        true,
{
    open_local_invariant!(inv => points_to => {
        assert(points_to.is_init());
        assert(points_to.value() % 2 == 0);

        let x = cell.take(Tracked(&mut points_to));
        assert(x % 2 == 0);

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

// use vstd::cell::spec::MemContents;
//   could not find `spec` in `cell`: spec

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1