use vstd::prelude::*;

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        old(orig).balance >= amount,
        old(dest).balance + amount < u64::MAX,
    ensures
        dest.balance == old(dest).balance + amount,
        orig.balance == old(orig).balance - amount,
{
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
}

/* TEST CODE BELOW */