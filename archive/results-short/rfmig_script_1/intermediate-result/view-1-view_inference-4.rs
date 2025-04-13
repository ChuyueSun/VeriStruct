
// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        amount <= orig.balance,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        old(perm).opt_value().is_Init(),
    ensures
        match old(perm).opt_value() {
            MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),
            _ => false
        },
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.opt_value().is_Uninit(), // or suitably Null to be initialized
    ensures
        perm.opt_value() === MemContents::Init(6),
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3
// VerusErrorType.Other: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scopeVerusErrorType.MismatchedType: mismatched typesVerusErrorType.Other: no method named `is_Uninit` found for enum `vstd::raw_ptr::MemContents` in the current scope
// {"$message_type":"diagnostic","message":"no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":897,"byte_end":904,"line_start":30,"line_end":30,"column_start":31,"column_end":38,"is_primary":true,"text":[{"text":"        old(perm).opt_value().is_Init(),","highlight_start":31,"highlight_end":38}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"there is a method `is_init` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":897,"byte_end":904,"line_start":30,"line_end":30,"column_start":31,"column_end":38,"is_primary":true,"text":[{"text":"        old(perm).opt_value().is_Init(),","highlight_start":31,"highlight_end":38}],"label":null,"suggested_replacement":"is_init","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f:30:31\n   |\n30 |         old(perm).opt_value().is_Init(),\n   |                               ^^^^^^^\n   |\nhelp: there is a method `is_init` with a similar name (notice the capitalization difference)\n   |\n30 |         old(perm).opt_value().is_init(),\n   |                               ~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":"Expected type did not match the received type.\n\nErroneous code examples:\n\n```compile_fail,E0308\nfn plus_one(x: i32) -> i32 {\n    x + 1\n}\n\nplus_one(\"Not a number\");\n//       ^^^^^^^^^^^^^^ expected `i32`, found `&str`\n\nif \"Not a bool\" {\n// ^^^^^^^^^^^^ expected `bool`, found `&str`\n}\n\nlet x: f32 = \"Not a float\";\n//     ---   ^^^^^^^^^^^^^ expected `f32`, found `&str`\n//     |\n//     expected due to this\n```\n\nThis error occurs when an expression was used in a place where the compiler\nexpected an expression of a different type. It can occur in several cases, the\nmost common being when calling a function and passing an argument which has a\ndifferent type than the matching type in the function declaration.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1033,"byte_end":1038,"line_start":33,"line_end":33,"column_start":76,"column_end":81,"is_primary":true,"text":[{"text":"            MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),","highlight_start":76,"highlight_end":81}],"label":"expected `u64`, found `int`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1015,"byte_end":1032,"line_start":33,"line_end":33,"column_start":58,"column_end":75,"is_primary":false,"text":[{"text":"            MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),","highlight_start":58,"highlight_end":75}],"label":"arguments to this enum variant are incorrect","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"the type constructed contains `builtin::int` due to the type of the argument passed","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1033,"byte_end":1038,"line_start":33,"line_end":33,"column_start":76,"column_end":81,"is_primary":false,"text":[{"text":"            MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),","highlight_start":76,"highlight_end":81}],"label":"this argument influences the type of `MemContents`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1015,"byte_end":1039,"line_start":33,"line_end":33,"column_start":58,"column_end":82,"is_primary":true,"text":[{"text":"            MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),","highlight_start":58,"highlight_end":82}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null},{"message":"tuple variant defined here","code":null,"level":"note","spans":[{"file_name":"/Users/runner/work/verus/verus/source/vstd/raw_ptr.rs","byte_start":4720,"byte_end":4724,"line_start":130,"line_end":130,"column_start":5,"column_end":9,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0308]: mismatched types\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f:33:76\n   |\n33 |             MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),\n   |                                                          ----------------- ^^^^^ expected `u64`, found `int`\n   |                                                          |\n   |                                                          arguments to this enum variant are incorrect\n   |\nhelp: the type constructed contains `builtin::int` due to the type of the argument passed\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f:33:58\n   |\n33 |             MemContents::Init(x) => perm.opt_value() === MemContents::Init(x + 1),\n   |                                                          ^^^^^^^^^^^^^^^^^^-----^\n   |                                                                            |\n   |                                                                            this argument influences the type of `MemContents`\nnote: tuple variant defined here\n  --> /Users/runner/work/verus/verus/source/vstd/raw_ptr.rs:130:5\n\n"}
// {"$message_type":"diagnostic","message":"no method named `is_Uninit` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1295,"byte_end":1304,"line_start":43,"line_end":43,"column_start":26,"column_end":35,"is_primary":true,"text":[{"text":"        perm.opt_value().is_Uninit(), // or suitably Null to be initialized","highlight_start":26,"highlight_end":35}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"there is a variant with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1295,"byte_end":1304,"line_start":43,"line_end":43,"column_start":26,"column_end":35,"is_primary":true,"text":[{"text":"        perm.opt_value().is_Uninit(), // or suitably Null to be initialized","highlight_start":26,"highlight_end":35}],"label":null,"suggested_replacement":"Uninit","suggestion_applicability":"HasPlaceholders","expansion":null}],"children":[],"rendered":null},{"message":"there is a method `is_uninit` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f","byte_start":1295,"byte_end":1304,"line_start":43,"line_end":43,"column_start":26,"column_end":35,"is_primary":true,"text":[{"text":"        perm.opt_value().is_Uninit(), // or suitably Null to be initialized","highlight_start":26,"highlight_end":35}],"label":null,"suggested_replacement":"is_uninit","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `is_Uninit` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpfkajn49f:43:26\n   |\n43 |         perm.opt_value().is_Uninit(), // or suitably Null to be initialized\n   |                          ^^^^^^^^^\n   |\nhelp: there is a variant with a similar name\n   |\n43 |         perm.opt_value().Uninit(), // or suitably Null to be initialized\n   |                          ~~~~~~\nhelp: there is a method `is_uninit` with a similar name (notice the capitalization difference)\n   |\n43 |         perm.opt_value().is_uninit(), // or suitably Null to be initialized\n   |                          ~~~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 3 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 3 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"Some errors have detailed explanations: E0308, E0599.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"Some errors have detailed explanations: E0308, E0599.\n"}
// {"$message_type":"diagnostic","message":"For more information about an error, try `rustc --explain E0308`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about an error, try `rustc --explain E0308`.\n"}
// 
// 