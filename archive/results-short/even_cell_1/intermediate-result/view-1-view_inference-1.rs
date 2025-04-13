use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        // Ensures that if the cell is initialized, its value is always even.
        points_to.is_init() ==> points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>)
    requires
        inv.can_open(),
        cell.id() == inv.resource_id(),
    ensures
        inv.can_open(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope
// {"$message_type":"diagnostic","message":"no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16","byte_start":506,"byte_end":514,"line_start":18,"line_end":18,"column_start":13,"column_end":21,"is_primary":true,"text":[{"text":"        inv.can_open(),","highlight_start":13,"highlight_end":21}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16:18:13\n   |\n18 |         inv.can_open(),\n   |             ^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16","byte_start":543,"byte_end":554,"line_start":19,"line_end":19,"column_start":26,"column_end":37,"is_primary":true,"text":[{"text":"        cell.id() == inv.resource_id(),","highlight_start":26,"highlight_end":37}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16:19:26\n   |\n19 |         cell.id() == inv.resource_id(),\n   |                          ^^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16","byte_start":582,"byte_end":590,"line_start":21,"line_end":21,"column_start":13,"column_end":21,"is_primary":true,"text":[{"text":"        inv.can_open(),","highlight_start":13,"highlight_end":21}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `can_open` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6_x5hj16:21:13\n   |\n21 |         inv.can_open(),\n   |             ^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6_x5hj16.long-type-10013452168364873968.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 