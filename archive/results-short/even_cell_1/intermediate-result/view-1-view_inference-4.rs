use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        !points_to.is_init() || points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>)
    requires
        inv.inv_is_valid(),
        cell.id() == inv.resource_id(),
    ensures
        inv.inv_is_valid(),
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
// VerusErrorType.Other: no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope
// {"$message_type":"diagnostic","message":"no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8","byte_start":428,"byte_end":440,"line_start":17,"line_end":17,"column_start":13,"column_end":25,"is_primary":true,"text":[{"text":"        inv.inv_is_valid(),","highlight_start":13,"highlight_end":25}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8:17:13\n   |\n17 |         inv.inv_is_valid(),\n   |             ^^^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8","byte_start":469,"byte_end":480,"line_start":18,"line_end":18,"column_start":26,"column_end":37,"is_primary":true,"text":[{"text":"        cell.id() == inv.resource_id(),","highlight_start":26,"highlight_end":37}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8:18:26\n   |\n18 |         cell.id() == inv.resource_id(),\n   |                          ^^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8","byte_start":508,"byte_end":520,"line_start":20,"line_end":20,"column_start":13,"column_end":25,"is_primary":true,"text":[{"text":"        inv.inv_is_valid(),","highlight_start":13,"highlight_end":25}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `inv_is_valid` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp6lp51pt8:20:13\n   |\n20 |         inv.inv_is_valid(),\n   |             ^^^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp6lp51pt8.long-type-11894064932664616028.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
//
//
