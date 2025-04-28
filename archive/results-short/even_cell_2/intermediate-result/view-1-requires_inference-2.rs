use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

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
        inv.inv(),
        cell.id() == inv.id(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: failed to resolve: use of undeclared type `MemContents`
// {"$message_type":"diagnostic","message":"failed to resolve: use of undeclared type `MemContents`","code":{"code":"E0433","explanation":"An undeclared crate, module, or type was used.\n\nErroneous code example:\n\n```compile_fail,E0433\nlet map = HashMap::new();\n// error: failed to resolve: use of undeclared type `HashMap`\n```\n\nPlease verify you didn't misspell the type/module's name or that you didn't\nforget to import it:\n\n```\nuse std::collections::HashMap; // HashMap has been imported.\nlet map: HashMap<u32, u32> = HashMap::new(); // So it can be used!\n```\n\nIf you've expected to use a crate name:\n\n```compile_fail\nuse ferris_wheel::BigO;\n// error: failed to resolve: use of undeclared crate or module `ferris_wheel`\n```\n\nMake sure the crate has been added as a dependency in `Cargo.toml`.\n\nTo use a module from your current crate, add the `crate::` prefix to the path.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpg0fnv6pn","byte_start":321,"byte_end":332,"line_start":12,"line_end":12,"column_start":46,"column_end":57,"is_primary":true,"text":[{"text":"        &&& points_to.mem_contents() matches MemContents::Init(val)","highlight_start":46,"highlight_end":57}],"label":"use of undeclared type `MemContents`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider importing this enum","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpg0fnv6pn","byte_start":0,"byte_end":0,"line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":true,"text":[],"label":null,"suggested_replacement":"use vstd::simple_pptr::MemContents;\n","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0433]: failed to resolve: use of undeclared type `MemContents`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpg0fnv6pn:12:46\n   |\n12 |         &&& points_to.mem_contents() matches MemContents::Init(val)\n   |                                              ^^^^^^^^^^^ use of undeclared type `MemContents`\n   |\nhelp: consider importing this enum\n   |\n1  + use vstd::simple_pptr::MemContents;\n   |\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0433`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0433`.\n"}
//
//
