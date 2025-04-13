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
        let c = *counter.borrow(Tracked(&*perm)),
        old(c) < u64::MAX,
    ensures
        *counter.borrow(Tracked(&*perm)) == old(c) + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> ()
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
// VerusErrorType.Other: expected expression, found `let` statement
// {"$message_type":"diagnostic","message":"expected expression, found `let` statement","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmps9941fb4","byte_start":798,"byte_end":801,"line_start":28,"line_end":28,"column_start":9,"column_end":12,"is_primary":true,"text":[{"text":"        let c = *counter.borrow(Tracked(&*perm)),","highlight_start":9,"highlight_end":12}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[{"message":"only supported directly in conditions of `if` and `while` expressions","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"error: expected expression, found `let` statement\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmps9941fb4:28:9\n   |\n28 |         let c = *counter.borrow(Tracked(&*perm)),\n   |         ^^^\n   |\n   = note: only supported directly in conditions of `if` and `while` expressions\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
// 
// 