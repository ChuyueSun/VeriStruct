use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        points_to.is_init()
        && points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>) -> (ret: ())
    requires
        inv.id() == cell.id(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.Other: no method named `id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: the method `view` exists for reference `&PCell<u8>`, but its trait bounds were not satisfied
// {"$message_type":"diagnostic","message":"no method named `id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmprhocahj3","byte_start":448,"byte_end":450,"line_start":18,"line_end":18,"column_start":13,"column_end":15,"is_primary":true,"text":[{"text":"        inv.id() == cell.id(),","highlight_start":13,"highlight_end":15}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmprhocahj3.long-type-13264347705151187127.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmprhocahj3:18:13\n   |\n18 |         inv.id() == cell.id(),\n   |             ^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmprhocahj3.long-type-13264347705151187127.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"the method `view` exists for reference `&PCell<u8>`, but its trait bounds were not satisfied","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmprhocahj3","byte_start":491,"byte_end":492,"line_start":20,"line_end":20,"column_start":13,"column_end":14,"is_primary":true,"text":[{"text":"        cell@ % 2 == 0,","highlight_start":13,"highlight_end":14}],"label":"method cannot be called on `&PCell<u8>` due to unsatisfied trait bounds","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the following trait bounds were not satisfied:\n`vstd::cell::PCell<u8>: vstd::string::View`\nwhich is required by `&vstd::cell::PCell<u8>: vstd::string::View`","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: the method `view` exists for reference `&PCell<u8>`, but its trait bounds were not satisfied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmprhocahj3:20:13\n   |\n20 |         cell@ % 2 == 0,\n   |             ^ method cannot be called on `&PCell<u8>` due to unsatisfied trait bounds\n   |\n   = note: the following trait bounds were not satisfied:\n           `vstd::cell::PCell<u8>: vstd::string::View`\n           which is required by `&vstd::cell::PCell<u8>: vstd::string::View`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
//
//
