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
        old(orig).balance >= amount,
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

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
    requires
        *counter.borrow(Tracked(&*perm)) < u64::MAX,
    ensures
        *counter.borrow(Tracked(&*perm)) <= u64::MAX,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        true,
    ensures
        *counter.borrow(Tracked(&perm)) == 6,
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: in requires, use `old(perm)` to refer to the pre-state of an &mut variable
// {"$message_type":"diagnostic","message":"in requires, use `old(perm)` to refer to the pre-state of an &mut variable","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmphq_omw6n","byte_start":905,"byte_end":909,"line_start":29,"line_end":29,"column_start":35,"column_end":39,"is_primary":true,"text":[{"text":"        *counter.borrow(Tracked(&*perm)) < u64::MAX,","highlight_start":35,"highlight_end":39}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: in requires, use `old(perm)` to refer to the pre-state of an &mut variable\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmphq_omw6n:29:35\n   |\n29 |         *counter.borrow(Tracked(&*perm)) < u64::MAX,\n   |                                   ^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 