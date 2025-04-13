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
        amount <= old(orig).balance,
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
        exists|val: u64| perm.opt_value() === MemContents::Init(val),
    ensures
        perm.opt_value() === MemContents::Init(old(perm).unwrap_init() + 1),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.opt_value() === MemContents::Uninit || perm.opt_value() === MemContents::Init(_),
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
// VerusErrorType.Other: in expressions, `_` can only be used on the left-hand side of an assignmentVerusErrorType.Other: no method named `unwrap_init` found for mutable reference `&mut vstd::simple_pptr::PointsTo<u64>` in the current scope
// {"$message_type":"diagnostic","message":"in expressions, `_` can only be used on the left-hand side of an assignment","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp8zl_awbk","byte_start":1316,"byte_end":1317,"line_start":39,"line_end":39,"column_start":92,"column_end":93,"is_primary":true,"text":[{"text":"        perm.opt_value() === MemContents::Uninit || perm.opt_value() === MemContents::Init(_),","highlight_start":92,"highlight_end":93}],"label":"`_` not allowed here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: in expressions, `_` can only be used on the left-hand side of an assignment\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp8zl_awbk:39:92\n   |\n39 |         perm.opt_value() === MemContents::Uninit || perm.opt_value() === MemContents::Init(_),\n   |                                                                                            ^ `_` not allowed here\n\n"}
// {"$message_type":"diagnostic","message":"no method named `unwrap_init` found for mutable reference `&mut vstd::simple_pptr::PointsTo<u64>` in the current scope","code":{"code":"E0599","explanation":"This error occurs when a method is used on a type which doesn't implement it:\n\nErroneous code example:\n\n```compile_fail,E0599\nstruct Mouth;\n\nlet x = Mouth;\nx.chocolate(); // error: no method named `chocolate` found for type `Mouth`\n               //        in the current scope\n```\n\nIn this case, you need to implement the `chocolate` method to fix the error:\n\n```\nstruct Mouth;\n\nimpl Mouth {\n    fn chocolate(&self) { // We implement the `chocolate` method here.\n        println!(\"Hmmm! I love chocolate!\");\n    }\n}\n\nlet x = Mouth;\nx.chocolate(); // ok!\n```\n"},"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp8zl_awbk","byte_start":1010,"byte_end":1021,"line_start":31,"line_end":31,"column_start":58,"column_end":69,"is_primary":true,"text":[{"text":"        perm.opt_value() === MemContents::Init(old(perm).unwrap_init() + 1),","highlight_start":58,"highlight_end":69}],"label":"method not found in `&mut PointsTo<u64>`","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0599]: no method named `unwrap_init` found for mutable reference `&mut vstd::simple_pptr::PointsTo<u64>` in the current scope\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp8zl_awbk:31:58\n   |\n31 |         perm.opt_value() === MemContents::Init(old(perm).unwrap_init() + 1),\n   |                                                          ^^^^^^^^^^^ method not found in `&mut PointsTo<u64>`\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 2 previous errors","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 2 previous errors\n\n"}
// {"$message_type":"diagnostic","message":"For more information about this error, try `rustc --explain E0599`.","code":null,"level":"failure-note","spans":[],"children":[],"rendered":"For more information about this error, try `rustc --explain E0599`.\n"}
// 
// 