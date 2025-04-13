use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

#[verifier::loop_isolation(false)]

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> (ret: ())
    requires
        old(orig).balance >= amount,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
        orig.balance + dest.balance == old(orig).balance + old(dest).balance,
        ret == (),
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
    ret
}

fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>) -> (ret: ())
    requires
        *counter.borrow(Tracked(&*perm)) < u64::MAX,
    ensures
        *counter.borrow(Tracked(&*perm)) <= u64::MAX,
        ret == (),
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
    ret
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>) -> (ret: ())
    ensures
        perm.opt_value() === MemContents::Init(6),
        ret == (),
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
    ret
}

} // verus!

// pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()
//   None: pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) -> ()

// Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 3