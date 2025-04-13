use vstd::prelude::*;
use vstd::invariant::*;
use vstd::cell::*;

verus!{

ghost struct EvenCell { }

impl InvariantPredicate<CellId, PointsTo<u8>> for EvenCell {
    open spec fn inv(cell_id: CellId, points_to: PointsTo<u8>) -> bool {
        points_to.is_init() && points_to.value() % 2 == 0
    }
}

impl LocalInvariant<CellId, PointsTo<u8>, EvenCell> {
    open spec fn view(&self) -> (CellId, u8) {
        (self.identifier(), self.global().value())
    }
}

fn add_2(cell: &PCell<u8>, Tracked(inv): Tracked<&LocalInvariant<CellId, PointsTo<u8>, EvenCell>>)
    requires
        cell.id() == inv.identifier(),
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: cannot define inherent `impl` for a type outside of the crate where the type is definedVerusErrorType.Other: no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `global` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scopeVerusErrorType.Other: no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope
// {"$message_type":"diagnostic","message":"cannot define inherent `impl` for a type outside of the crate where the type is defined","code":{"code":"E0390","explanation":"A method or constant was implemented on a primitive type.\n\nErroneous code example:\n\n```compile_fail,E0390\nstruct Foo {\n    x: i32\n}\n\nimpl *mut Foo {}\n// error: cannot define inherent `impl` for primitive types\n```\n\nThis isn't allowed, but using a trait to implement a method or constant\nis a good solution.\nExample:\n\n```\nstruct Foo {\n    x: i32\n}\n\ntrait Bar {\n    fn bar();\n}\n\nimpl Bar for *mut Foo {\n    fn bar() {} // ok!\n}\n```\n\nInstead of defining an inherent implementation on a reference, you could also\nmove the reference inside the implementation:\n\n```compile_fail,E0390\nstruct Foo;\n\nimpl &Foo { // error: no nominal type found for inherent implementation\n    fn bar(self, other: Self) {}\n}\n```\n\nbecomes\n\n```\nstruct Foo;\n\nimpl Foo {\n    fn bar(&self, other: &Self) {}\n}\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw","byte_start":303,"byte_end":354,"line_start":15,"line_end":15,"column_start":1,"column_end":52,"is_primary":true,"text":[{"text":"impl LocalInvariant<CellId, PointsTo<u8>, EvenCell> {","highlight_start":1,"highlight_end":52}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"consider moving this inherent impl into the crate defining the type if possible","code":null,"level":"help","spans":[],"children":[],"rendered":null},{"message":"alternatively add `#[rustc_has_incoherent_inherent_impls]` to the type and `#[rustc_allow_incoherent_impl]` to the relevant impl items","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw","byte_start":303,"byte_end":354,"line_start":15,"line_end":15,"column_start":1,"column_end":52,"is_primary":true,"text":[{"text":"impl LocalInvariant<CellId, PointsTo<u8>, EvenCell> {","highlight_start":1,"highlight_end":52}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0390]: cannot define inherent `impl` for a type outside of the crate where the type is defined\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw:15:1\n   |\n15 | impl LocalInvariant<CellId, PointsTo<u8>, EvenCell> {\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n   |\n   = help: consider moving this inherent impl into the crate defining the type if possible\nhelp: alternatively add `#[rustc_has_incoherent_inherent_impls]` to the type and `#[rustc_allow_incoherent_impl]` to the relevant impl items\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw:15:1\n   |\n15 | impl LocalInvariant<CellId, PointsTo<u8>, EvenCell> {\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw","byte_start":418,"byte_end":428,"line_start":17,"line_end":17,"column_start":15,"column_end":25,"is_primary":true,"text":[{"text":"        (self.identifier(), self.global().value())","highlight_start":15,"highlight_end":25}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmpsfa_jvrw.long-type-11733122577481851810.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw:17:15\n   |\n17 |         (self.identifier(), self.global().value())\n   |               ^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmpsfa_jvrw.long-type-11733122577481851810.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `global` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw","byte_start":437,"byte_end":443,"line_start":17,"line_end":17,"column_start":34,"column_end":40,"is_primary":true,"text":[{"text":"        (self.identifier(), self.global().value())","highlight_start":34,"highlight_end":40}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmpsfa_jvrw.long-type-11733122577481851810.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `global` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw:17:34\n   |\n17 |         (self.identifier(), self.global().value())\n   |                                  ^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmpsfa_jvrw.long-type-11733122577481851810.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw","byte_start":601,"byte_end":611,"line_start":23,"line_end":23,"column_start":26,"column_end":36,"is_primary":true,"text":[{"text":"        cell.id() == inv.identifier(),","highlight_start":26,"highlight_end":36}],"label":"method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the full type name has been written to 'tmpsfa_jvrw.long-type-3854070562837963013.txt'","code":null,"level":"note","spans":[],"children":[],"rendered":null},{"message":"consider using `--verbose` to print the full type name to the console","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `identifier` found for reference `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpsfa_jvrw:23:26\n   |\n23 |         cell.id() == inv.identifier(),\n   |                          ^^^^^^^^^^ method not found in `&LocalInvariant<CellId, PointsTo<u8>, EvenCell>`\n   |\n   = note: the full type name has been written to 'tmpsfa_jvrw.long-type-3854070562837963013.txt'\n   = note: consider using `--verbose` to print the full type name to the console\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0390, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0390, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0390`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0390`.\n"}
// 
// 