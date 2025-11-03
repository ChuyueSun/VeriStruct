use vstd::prelude::*;

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        old(orig).balance >= amount,
        old(dest).balance + amount <= u64::MAX,
    ensures
        orig.balance == old(orig).balance - amount,
        dest.balance == old(dest).balance + amount,
{
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
}

/* TEST CODE BELOW */

pub fn test1(init_balance: u64, transfer_amount: u64)
requires
    init_balance >= transfer_amount,
    transfer_amount > 0,
    init_balance < u64::MAX - transfer_amount,
{
    let mut acc1 = Account { balance: init_balance };
    let mut acc2 = Account { balance: 0 };
    transfer(&mut acc1, &mut acc2, transfer_amount);
    // assert(acc1.balance == init_balance - transfer_amount);
    // assert(acc2.balance == transfer_amount);
}

pub fn test2(init_balance: u64, transfer_amount: u64)
requires
    init_balance >= transfer_amount,
    transfer_amount > 0,
    init_balance < u64::MAX - transfer_amount,
{
    let mut acc1 = Account { balance: init_balance };
    let mut acc2 = Account { balance: 0 };
    transfer(&mut acc1, &mut acc2, transfer_amount);
    assert(acc1.balance == init_balance - transfer_amount);
    // assert(acc2.balance == transfer_amount);
}

pub fn test3(init_balance: u64, transfer_amount: u64)
requires
    init_balance >= transfer_amount,
    transfer_amount > 0,
    init_balance < u64::MAX - transfer_amount,
{
    let mut acc1 = Account { balance: init_balance };
    let mut acc2 = Account { balance: 0 };
    transfer(&mut acc1, &mut acc2, transfer_amount);
    assert(acc1.balance == init_balance - transfer_amount);
    assert(acc2.balance == transfer_amount);
}

pub fn main() {
}
