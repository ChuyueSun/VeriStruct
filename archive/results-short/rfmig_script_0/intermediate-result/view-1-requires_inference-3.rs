// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()
    requires
        amount <= old(orig).balance,
    ensures
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> ()
    requires
        old(perm).opt_value().is_Init(),
    ensures
        perm.opt_value().is_Init(),
        perm.opt_value().get_Init() == old(perm).opt_value().get_Init() + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> ()
    requires
        true,
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// VerusErrorType.Other: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scopeVerusErrorType.Other: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scopeVerusErrorType.Other: no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scopeVerusErrorType.Other: no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope
// {"$message_type":"diagnostic","message":"no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":913,"byte_end":920,"line_start":29,"line_end":29,"column_start":31,"column_end":38,"is_primary":true,"text":[{"text":"        old(perm).opt_value().is_Init(),","highlight_start":31,"highlight_end":38}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"there is a method `is_init` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":913,"byte_end":920,"line_start":29,"line_end":29,"column_start":31,"column_end":38,"is_primary":true,"text":[{"text":"        old(perm).opt_value().is_Init(),","highlight_start":31,"highlight_end":38}],"label":null,"suggested_replacement":"is_init","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv:29:31\n   |\n29 |         old(perm).opt_value().is_Init(),\n   |                               ^^^^^^^\n   |\nhelp: there is a method `is_init` with a similar name (notice the capitalization difference)\n   |\n29 |         old(perm).opt_value().is_init(),\n   |                               ~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":961,"byte_end":968,"line_start":31,"line_end":31,"column_start":26,"column_end":33,"is_primary":true,"text":[{"text":"        perm.opt_value().is_Init(),","highlight_start":26,"highlight_end":33}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"there is a method `is_init` with a similar name","code":null,"level":"help","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":961,"byte_end":968,"line_start":31,"line_end":31,"column_start":26,"column_end":33,"is_primary":true,"text":[{"text":"        perm.opt_value().is_Init(),","highlight_start":26,"highlight_end":33}],"label":null,"suggested_replacement":"is_init","suggestion_applicability":"MaybeIncorrect","expansion":null}],"children":[],"rendered":null}],"rendered":"error[E0599]: no method named `is_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv:31:26\n   |\n31 |         perm.opt_value().is_Init(),\n   |                          ^^^^^^^\n   |\nhelp: there is a method `is_init` with a similar name (notice the capitalization difference)\n   |\n31 |         perm.opt_value().is_init(),\n   |                          ~~~~~~~\n\n"}
// {"$message_type":"diagnostic","message":"no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":997,"byte_end":1005,"line_start":32,"line_end":32,"column_start":26,"column_end":34,"is_primary":true,"text":[{"text":"        perm.opt_value().get_Init() == old(perm).opt_value().get_Init() + 1,","highlight_start":26,"highlight_end":34}],"label":"method not found in `MemContents<u64>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv:32:26\n   |\n32 |         perm.opt_value().get_Init() == old(perm).opt_value().get_Init() + 1,\n   |                          ^^^^^^^^ method not found in `MemContents<u64>`\n\n"}
// {"$message_type":"diagnostic","message":"no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv","byte_start":1033,"byte_end":1041,"line_start":32,"line_end":32,"column_start":62,"column_end":70,"is_primary":true,"text":[{"text":"        perm.opt_value().get_Init() == old(perm).opt_value().get_Init() + 1,","highlight_start":62,"highlight_end":70}],"label":"method not found in `MemContents<u64>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `get_Init` found for enum `vstd::raw_ptr::MemContents` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpz0sv9ozv:32:62\n   |\n32 |         perm.opt_value().get_Init() == old(perm).opt_value().get_Init() + 1,\n   |                                                              ^^^^^^^^ method not found in `MemContents<u64>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 4 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 4 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
//
//
