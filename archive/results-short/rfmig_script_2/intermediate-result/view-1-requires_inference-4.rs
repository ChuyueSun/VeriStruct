#![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

impl Account {
    pub open spec fn view(&self) -> (nat) {
        (self.balance as nat,)
    }

    #[verifier::type_invariant]
    pub closed spec fn inv(&self) -> bool {
        true
    }
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()
    requires
        orig.balance >= amount,
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
        counter ~~ perm,
    ensures
        *counter.borrow(Tracked(&*perm)) == old(*counter.borrow(Tracked(&*perm))) + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        counter ~~ perm,
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
// VerusErrorType.Other: expected curly braces
// {"$message_type":"diagnostic","message":"expected curly braces","code":null,"level":"error","spans":[{"file_name":"/var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpc_vd9i2d","byte_start":1018,"byte_end":1019,"line_start":41,"line_end":41,"column_start":17,"column_end":18,"is_primary":true,"text":[{"text":"        counter ~~ perm,","highlight_start":17,"highlight_end":18}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected curly braces\n  --> /var/folders/nh/_8qdng_n3357qvdjjrx5mchw0000gn/T/tmpc_vd9i2d:41:17\n   |\n41 |         counter ~~ perm,\n   |                 ^\n\n"}
// {"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n\n"}
//
//
