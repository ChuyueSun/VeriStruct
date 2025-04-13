// #![allow(unused_imports, unused_macros, non_camel_case_types)]
#![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub open spec fn View(a: &Account) -> (nat,) {
    (a.balance as nat,)
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
        perm.opt_value() === MemContents::Init(old_val),
    ensures
        perm.opt_value() === MemContents::Init(old_val + 1),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.opt_value() === MemContents::Uninit,
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
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// VerusErrorType.Other: cannot find value `old_val` in this scopeVerusErrorType.Other: cannot find value `old_val` in this scope
// {"$message_type":"diagnostic","message":"cannot find value `old_val` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp0z_7r2_7","byte_start":987,"byte_end":994,"line_start":34,"line_end":34,"column_start":48,"column_end":55,"is_primary":true,"text":[{"text":"        perm.opt_value() === MemContents::Init(old_val),","highlight_start":48,"highlight_end":55}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `old_val` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp0z_7r2_7:34:48\n   |\n34 |         perm.opt_value() === MemContents::Init(old_val),\n   |                                                ^^^^^^^ not found in this scope\n\n"}
// {"$message_type":"diagnostic","message":"cannot find value `old_val` in this scope","code":{"code":"E0425","explanation":"An unresolved name was used.\n\nErroneous code examples:\n\n```compile_fail,E0425\nsomething_that_doesnt_exist::foo;\n// error: unresolved name `something_that_doesnt_exist::foo`\n\n// or:\n\ntrait Foo {\n    fn bar() {\n        Self; // error: unresolved name `Self`\n    }\n}\n\n// or:\n\nlet x = unknown_variable;  // error: unresolved name `unknown_variable`\n```\n\nPlease verify that the name wasn't misspelled and ensure that the\nidentifier being referred to is valid for the given situation. Example:\n\n```\nenum something_that_does_exist {\n    Foo,\n}\n```\n\nOr:\n\n```\nmod something_that_does_exist {\n    pub static foo : i32 = 0i32;\n}\n\nsomething_that_does_exist::foo; // ok!\n```\n\nOr:\n\n```\nlet unknown_variable = 12u32;\nlet x = unknown_variable; // ok!\n```\n\nIf the item is not defined in the current module, it must be imported using a\n`use` statement, like so:\n\n```\n# mod foo { pub fn bar() {} }\n# fn main() {\nuse foo::bar;\nbar();\n# }\n```\n\nIf the item you are importing is not defined in some super-module of the\ncurrent module, then it must also be declared as public (e.g., `pub fn`).\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp0z_7r2_7","byte_start":1056,"byte_end":1063,"line_start":36,"line_end":36,"column_start":48,"column_end":55,"is_primary":true,"text":[{"text":"        perm.opt_value() === MemContents::Init(old_val + 1),","highlight_start":48,"highlight_end":55}],"label":"not found in this scope","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `old_val` in this scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp0z_7r2_7:36:48\n   |\n36 |         perm.opt_value() === MemContents::Init(old_val + 1),\n   |                                                ^^^^^^^ not found in this scope\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0425`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0425`.\n"}
// 
// 