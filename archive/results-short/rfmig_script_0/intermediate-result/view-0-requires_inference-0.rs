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
        exists |x: u64| old(perm).opt_value() === MemContents::Init(x),
    ensures
        forall |x: u64| old(perm).opt_value() === MemContents::Init(x)
            ==> perm.opt_value() === MemContents::Init(( x + 1 ) as u64),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
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

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.Other: Could not automatically infer triggers for this quantifer.  Use #[trigger] annotations to manually mark trigger terms instead.
// {"$message_type":"diagnostic","message":"Could not automatically infer triggers for this quantifer.  Use #[trigger] annotations to manually mark trigger terms instead.","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyh79tjni","byte_start":964,"byte_end":1099,"line_start":32,"line_end":33,"column_start":9,"column_end":73,"is_primary":true,"text":[{"text":"        forall |x: u64| old(perm).opt_value() === MemContents::Init(x)","highlight_start":9,"highlight_end":71},{"text":"            ==> perm.opt_value() === MemContents::Init(( x + 1 ) as u64),","highlight_start":1,"highlight_end":73}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: Could not automatically infer triggers for this quantifer.  Use #[trigger] annotations to manually mark trigger terms instead.\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpyh79tjni:32:9\n   |\n32 | /         forall |x: u64| old(perm).opt_value() === MemContents::Init(x)\n33 | |             ==> perm.opt_value() === MemContents::Init(( x + 1 ) as u64),\n   | |________________________________________________________________________^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 