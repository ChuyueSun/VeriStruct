use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()
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

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> ()
    requires
        *counter.borrow(Tracked(&*perm)) < u64::MAX,
    ensures
        *counter.borrow(Tracked(&*perm)) <= u64::MAX,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> ()
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
// VerusErrorType.Other: the return value must be named in a function with an ensures clause
// {"$message_type":"diagnostic","message":"the return value must be named in a function with an ensures clause","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp877c65y3","byte_start":119,"byte_end":193,"line_start":12,"line_end":12,"column_start":1,"column_end":75,"is_primary":true,"text":[{"text":"pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()","highlight_start":1,"highlight_end":75}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: the return value must be named in a function with an ensures clause\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmp877c65y3:12:1\n   |\n12 | pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()\n   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 