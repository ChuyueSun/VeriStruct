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
pub fn main() {
    // Test case 1: Normal transfer
    let mut acc1 = Account { balance: 100 };
    let mut acc2 = Account { balance: 50 };
    transfer(&mut acc1, &mut acc2, 30);
    assert(acc1.balance == 70);
    assert(acc2.balance == 80);

    // Test case 2: Another valid transfer scenario
    let mut acc3 = Account { balance: 200 };
    let mut acc4 = Account { balance: 100 };
    transfer(&mut acc3, &mut acc4, 50);
    assert(acc3.balance == 150);
    assert(acc4.balance == 150);
}
} // verus!