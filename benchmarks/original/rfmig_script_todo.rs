// #![allow(unused_imports, unused_macros, non_camel_case_types)] #![feature(fmt_internals)]
use vstd::prelude::*;
use vstd::simple_pptr::*;

fn main() {}

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
// TODO: add requires and ensures
{
    let accounts_pre: Ghost<(Account, Account)> = Ghost((*orig, *dest));
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
    assert(orig.balance + dest.balance == accounts_pre@.0.balance + accounts_pre@.1.balance);
}


fn increment(counter: PPtr<u64>, Tracked(perm): Tracked<&mut PointsTo<u64>>)
// TODO: add requires and ensures
{
    let cur_i: u64 = *counter.borrow(Tracked(&*perm));
    counter.replace(Tracked(perm), cur_i + 1);
}

fn start_thread(counter: PPtr<u64>, Tracked(perm): Tracked<PointsTo<u64>>)
// TODO: add requires and ensures
{
    let tracked mut perm: PointsTo<u64> = perm;
    counter.put(Tracked(&mut perm), 5);
    assert(perm.opt_value() === MemContents::Init(5));
    increment(counter, Tracked(&mut perm));
    assert(perm.opt_value() === MemContents::Init(6));
}

} // verus!
