use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        points_to.is_init() && points_to.value() % 2 == 0
    }

    open spec fn view(cell_id: CellId, points_to: PointsTo<u8>) -> (CellId, nat) {
        (cell_id, points_to.value() as nat)
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>) -> (ret: ())
    requires
        cell.id() == inv.id(),
    ensures
        cell@ % 2 == 0,
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: method `view` is not a member of trait `InvariantPredicate`
// {"$message_type":"diagnostic","message":"method `view` is not a member of trait `InvariantPredicate`","code":{"code":"E0407","explanation":"A definition of a method not in the implemented trait was given in a trait\nimplementation.\n\nErroneous code example:\n\n```compile_fail,E0407\ntrait Foo {\n    fn a();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n    fn b() {} // error: method `b` is not a member of trait `Foo`\n}\n```\n\nPlease verify you didn't misspell the method name and you used the correct\ntrait. First example:\n\n```\ntrait Foo {\n    fn a();\n    fn b();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n    fn b() {} // ok!\n}\n```\n\nSecond example:\n\n```\ntrait Foo {\n    fn a();\n}\n\nstruct Bar;\n\nimpl Foo for Bar {\n    fn a() {}\n}\n\nimpl Bar {\n    fn b() {}\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6k3ptr_9","byte_start":315,"byte_end":433,"line_start":14,"line_end":16,"column_start":15,"column_end":6,"is_primary":true,"text":[{"text":"    open spec fn view(cell_id: CellId, points_to: PointsTo<u8>) -> (CellId, nat) {","highlight_start":15,"highlight_end":83},{"text":"        (cell_id, points_to.value() as nat)","highlight_start":1,"highlight_end":44},{"text":"    }","highlight_start":1,"highlight_end":6}],"label":"not a member of trait `InvariantPredicate`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0407]: method `view` is not a member of trait `InvariantPredicate`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6k3ptr_9:14:15\n   |\n14 |       open spec fn view(cell_id: CellId, points_to: PointsTo<u8>) -> (Ce...\n   |  _______________^\n15 | |         (cell_id, points_to.value() as nat)\n16 | |     }\n   | |_____^ not a member of trait `InvariantPredicate`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0407`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0407`.\n"}
//
//
