use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        // Ensures that if the cell is initialized, its value is even
        points_to.is_init() ==> points_to.value() % 2 == 0
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>)
    requires
        inv.inv(),
        cell.id() == inv.resource_id(),
    ensures
        inv.inv(),
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
// VerusErrorType.Other: this method takes 1 argument but 0 arguments were suppliedVerusErrorType.Other: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: this method takes 1 argument but 0 arguments were supplied
// {"$message_type":"diagnostic","message":"this method takes 1 argument but 0 arguments were supplied","code":{"code":"E0061","explanation":"An invalid number of arguments was passed when calling a function.\n\nErroneous code example:\n\n```compile_fail,E0061\nfn f(u: i32) {}\n\nf(); // error!\n```\n\nThe number of arguments passed to a function must match the number of arguments\nspecified in the function signature.\n\nFor example, a function like:\n\n```\nfn f(a: u16, b: &str) {}\n```\n\nMust always be called with exactly two arguments, e.g., `f(2, \"test\")`.\n\nNote that Rust does not have a notion of optional function arguments or\nvariadic functions (except for its C-FFI).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":501,"byte_end":503,"line_start":18,"line_end":18,"column_start":16,"column_end":18,"is_primary":false,"text":[{"text":"        inv.inv(),","highlight_start":16,"highlight_end":18}],"label":"argument #1 of type `vstd::cell::PointsTo<u8>` is missing","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":498,"byte_end":501,"line_start":18,"line_end":18,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"        inv.inv(),","highlight_start":13,"highlight_end":16}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":9340,"byte_end":9343,"line_start":191,"line_end":191,"column_start":30,"column_end":33,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":{"span":{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":10472,"byte_end":10511,"line_start":229,"line_end":229,"column_start":1,"column_end":40,"is_primary":false,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},"macro_decl_name":"declare_invariant_impl!","def_site_span":{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":8318,"byte_end":10428,"line_start":168,"line_end":226,"column_start":1,"column_end":2,"is_primary":false,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}}],"children":[],"rendered":null},{"message":"provide the argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":501,"byte_end":503,"line_start":18,"line_end":18,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        inv.inv(),","highlight_start":16,"highlight_end":18}],"label":null,"suggested_replacement":"(/* vstd::cell::PointsTo<u8> */)","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0061]: this method takes 1 argument but 0 arguments were supplied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8:18:13\n   |\n18 |         inv.inv(),\n   |             ^^^-- argument #1 of type `vstd::cell::PointsTo<u8>` is missing\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/invariant.rs:229:1\n   = note: this error originates in the macro `declare_invariant_impl` (in Nightly builds, run with -Z macro-backtrace for more info)\nhelp: provide the argument\n   |\n18 |         inv.inv(/* vstd::cell::PointsTo<u8> */),\n   |                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":530,"byte_end":541,"line_start":19,"line_end":19,"column_start":26,"column_end":37,"is_primary":true,"text":[{"text":"        cell.id() == inv.resource_id(),","highlight_start":26,"highlight_end":37}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmp31pv6s_8.long-type-9373206462239374517.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `resource_id` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8:19:26\n   |\n19 |         cell.id() == inv.resource_id(),\n   |                          ^^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmp31pv6s_8.long-type-9373206462239374517.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"this method takes 1 argument but 0 arguments were supplied","code":{"code":"E0061","explanation":"An invalid number of arguments was passed when calling a function.\n\nErroneous code example:\n\n```compile_fail,E0061\nfn f(u: i32) {}\n\nf(); // error!\n```\n\nThe number of arguments passed to a function must match the number of arguments\nspecified in the function signature.\n\nFor example, a function like:\n\n```\nfn f(a: u16, b: &str) {}\n```\n\nMust always be called with exactly two arguments, e.g., `f(2, \"test\")`.\n\nNote that Rust does not have a notion of optional function arguments or\nvariadic functions (except for its C-FFI).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":572,"byte_end":574,"line_start":21,"line_end":21,"column_start":16,"column_end":18,"is_primary":false,"text":[{"text":"        inv.inv(),","highlight_start":16,"highlight_end":18}],"label":"argument #1 of type `vstd::cell::PointsTo<u8>` is missing","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":569,"byte_end":572,"line_start":21,"line_end":21,"column_start":13,"column_end":16,"is_primary":true,"text":[{"text":"        inv.inv(),","highlight_start":13,"highlight_end":16}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"method defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":9340,"byte_end":9343,"line_start":191,"line_end":191,"column_start":30,"column_end":33,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":{"span":{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":10472,"byte_end":10511,"line_start":229,"line_end":229,"column_start":1,"column_end":40,"is_primary":false,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null},"macro_decl_name":"declare_invariant_impl!","def_site_span":{"file_name":"/Users/runner/work/verus/verus/source/vstd/invariant.rs","byte_start":8318,"byte_end":10428,"line_start":168,"line_end":226,"column_start":1,"column_end":2,"is_primary":false,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}}],"children":[],"rendered":null},{"message":"provide the argument","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8","byte_start":572,"byte_end":574,"line_start":21,"line_end":21,"column_start":16,"column_end":18,"is_primary":true,"text":[{"text":"        inv.inv(),","highlight_start":16,"highlight_end":18}],"label":null,"suggested_replacement":"(/* vstd::cell::PointsTo<u8> */)","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0061]: this method takes 1 argument but 0 arguments were supplied\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp31pv6s_8:21:13\n   |\n21 |         inv.inv(),\n   |             ^^^-- argument #1 of type `vstd::cell::PointsTo<u8>` is missing\n   |\nnote: method defined here\n  --> /Users/runner/work/verus/verus/source/vstd/invariant.rs:229:1\n   = note: this error originates in the macro `declare_invariant_impl` (in Nightly builds, run with -Z macro-backtrace for more info)\nhelp: provide the argument\n   |\n21 |         inv.inv(/* vstd::cell::PointsTo<u8> */),\n   |                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0061, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0061, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0061`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0061`.\n"}
//
//
