use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

#[verifier::loop_isolation(false)]

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> (res: ())
    requires
        old(orig).balance >= amount,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
        res == (),
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
    ()
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> (res: ())
    requires
        *counter.borrow(Tracked(old(perm))) < u64::MAX,
    ensures
        *counter.borrow(Tracked(&*perm)) <= u64::MAX,
        res == (),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
    ()
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> (res: ())
    ensures
        perm.opt_value() === MemContents::Init(6),
        res == (),
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
    ()
}

} // verus!
// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 1
// VerusErrorType.CannotCallFunc: cannot call function `vstd::simple_pptr::impl&%4::borrow` with mode exec
// {"$message_type":"diagnostic","message":"cannot call function `vstd::simple_pptr::impl&%4::borrow` with mode exec","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnrurh0wi","byte_start":875,"byte_end":909,"line_start":32,"line_end":32,"column_start":10,"column_end":44,"is_primary":true,"text":[{"text":"        *counter.borrow(Tracked(old(perm))) < u64::MAX,","highlight_start":10,"highlight_end":44}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: cannot call function `vstd::simple_pptr::impl&%4::borrow` with mode exec\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpnrurh0wi:32:10\n   |\n32 |         *counter.borrow(Tracked(old(perm))) < u64::MAX,\n   |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 