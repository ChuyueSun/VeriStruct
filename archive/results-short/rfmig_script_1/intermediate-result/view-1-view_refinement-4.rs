#![allow(unused_imports, unused_macros, non_camel_case_types)]
#![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

impl Account {
    pub open spec fn View(&self) -> (nat) {
        (self.balance as nat)
    }
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
        counter@.kind == PPtrKind::Valid,
        old(counter@) == old(perm@).id(),
    ensures
        *counter == old(*counter) + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        counter@.kind == PPtrKind::Valid,
        old(counter@) == old(perm@).id(),
    ensures
        *counter == 6,
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.Other: failed to resolve: use of undeclared type `PPtrKind`VerusErrorType.Other: failed to resolve: use of undeclared type `PPtrKind`
// {"$message_type":"diagnostic","message":"failed to resolve: use of undeclared type `PPtrKind`","code":{"code":"E0433","explanation":"An undeclared crate, module, or type was used.\n\nErroneous code example:\n\n```compile_fail,E0433\nlet map = HashMap::new();\n// error: failed to resolve: use of undeclared type `HashMap`\n```\n\nPlease verify you didn't misspell the type/module's name or that you didn't\nforget to import it:\n\n```\nuse std::collections::HashMap; // HashMap has been imported.\nlet map: HashMap<u32, u32> = HashMap::new(); // So it can be used!\n```\n\nIf you've expected to use a crate name:\n\n```compile_fail\nuse ferris_wheel::BigO;\n// error: failed to resolve: use of undeclared crate or module `ferris_wheel`\n```\n\nMake sure the crate has been added as a dependency in `Cargo.toml`.\n\nTo use a module from your current crate, add the `crate::` prefix to the path.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpcpl9jc3g","byte_start":986,"byte_end":994,"line_start":36,"line_end":36,"column_start":26,"column_end":34,"is_primary":true,"text":[{"text":"        counter@.kind == PPtrKind::Valid,","highlight_start":26,"highlight_end":34}],"label":"use of undeclared type `PPtrKind`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0433]: failed to resolve: use of undeclared type `PPtrKind`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpcpl9jc3g:36:26\n   |\n36 |         counter@.kind == PPtrKind::Valid,\n   |                          ^^^^^^^^ use of undeclared type `PPtrKind`\n\n"}
// {"$message_type":"diagnostic","message":"failed to resolve: use of undeclared type `PPtrKind`","code":{"code":"E0433","explanation":"An undeclared crate, module, or type was used.\n\nErroneous code example:\n\n```compile_fail,E0433\nlet map = HashMap::new();\n// error: failed to resolve: use of undeclared type `HashMap`\n```\n\nPlease verify you didn't misspell the type/module's name or that you didn't\nforget to import it:\n\n```\nuse std::collections::HashMap; // HashMap has been imported.\nlet map: HashMap<u32, u32> = HashMap::new(); // So it can be used!\n```\n\nIf you've expected to use a crate name:\n\n```compile_fail\nuse ferris_wheel::BigO;\n// error: failed to resolve: use of undeclared crate or module `ferris_wheel`\n```\n\nMake sure the crate has been added as a dependency in `Cargo.toml`.\n\nTo use a module from your current crate, add the `crate::` prefix to the path.\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpcpl9jc3g","byte_start":1316,"byte_end":1324,"line_start":47,"line_end":47,"column_start":26,"column_end":34,"is_primary":true,"text":[{"text":"        counter@.kind == PPtrKind::Valid,","highlight_start":26,"highlight_end":34}],"label":"use of undeclared type `PPtrKind`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0433]: failed to resolve: use of undeclared type `PPtrKind`\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpcpl9jc3g:47:26\n   |\n47 |         counter@.kind == PPtrKind::Valid,\n   |                          ^^^^^^^^ use of undeclared type `PPtrKind`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0433`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0433`.\n"}
//
//
