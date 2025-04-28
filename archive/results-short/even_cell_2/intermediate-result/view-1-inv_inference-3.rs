use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        &&& points_to.is_init()
        &&& points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>)
// TODO: add requires and ensures
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.PreCondFail: precondition not satisfiedVerusErrorType.PreCondFail: precondition not satisfied
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/cell.rs","byte_start":5931,"byte_end":5961,"line_start":193,"line_end":193,"column_start":13,"column_end":43,"is_primary":false,"text":[{"text":"            self.id() === old(perm)@.pcell,","highlight_start":13,"highlight_end":43}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpoczu3i5n","byte_start":597,"byte_end":631,"line_start":23,"line_end":23,"column_start":17,"column_end":51,"is_primary":true,"text":[{"text":"        let x = cell.take(Tracked(&mut points_to));","highlight_start":17,"highlight_end":51}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpoczu3i5n:23:17\n    |\n23  |         let x = cell.take(Tracked(&mut points_to));\n    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/cell.rs:193:13\n    |\n193 |             self.id() === old(perm)@.pcell,\n    |             ------------------------------ failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"precondition not satisfied","code":null,"level":"error","spans":[{"file_name":"/Users/syc/Desktop/verus/vstd/cell.rs","byte_start":5493,"byte_end":5547,"line_start":178,"line_end":178,"column_start":13,"column_end":67,"is_primary":false,"text":[{"text":"            old(perm)@ === pcell_opt![ self.id() => Option::None ],","highlight_start":13,"highlight_end":67}],"label":"failed precondition","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpoczu3i5n","byte_start":728,"byte_end":771,"line_start":28,"line_end":28,"column_start":9,"column_end":52,"is_primary":true,"text":[{"text":"        cell.put(Tracked(&mut points_to), x_plus_2);","highlight_start":9,"highlight_end":52}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: precondition not satisfied\n   --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpoczu3i5n:28:9\n    |\n28  |         cell.put(Tracked(&mut points_to), x_plus_2);\n    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n    |\n   ::: /Users/syc/Desktop/verus/vstd/cell.rs:178:13\n    |\n178 |             old(perm)@ === pcell_opt![ self.id() => Option::None ],\n    |             ------------------------------------------------------ failed precondition\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
//
//
