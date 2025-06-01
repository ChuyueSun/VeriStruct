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
        old(perm).pptr() === counter,
        old(perm).is_init(),
    ensures
        perm.pptr() === old(*perm).pptr(),
        perm.is_init(),
        perm.value() == old(*perm).value() + 1,
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
    requires
        perm.pptr() === counter,
        perm.is_uninit(),
    ensures
        perm.pptr() === old(( perm ) as &mut _).pptr(),
        perm.is_init(),
        perm.value() == old(perm).value() + 1,
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!

// Repair Round 4 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 9
// Verified: -1, Errors: 999, Verus Errors: 9