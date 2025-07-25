use vstd::prelude::*;

verus! {

pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64)
    requires
        orig.balance >= amount,
        dest.balance <= u64::MAX - amount,
    ensures
        orig.balance == old(orig.balance) - amount,
        dest.balance == old(dest.balance) + amount,
{
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
}

/* TEST CODE BELOW */

pub fn main() {
    // Test: test_transfer_valid
    {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 20);
        assert(orig.balance == 80);
        assert(dest.balance == 70);
    }

    // Test: test_transfer_zero
    {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 0);
        assert(orig.balance == 100);
        assert(dest.balance == 50);
    }

    // Test: test_transfer_full_balance
    {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 200 };
        transfer(&mut orig, &mut dest, 100);
        assert(orig.balance == 0);
        assert(dest.balance == 300);
    }

    // Test: test_transfer_insufficient_funds
    {
        let mut orig = Account { balance: 10 };
        let mut dest = Account { balance: 20 };
        // Precondition orig.balance >= amount is not met for amount = 15.
        // Thus, the transfer call is not permitted; we assert the violation.
        assert(!(orig.balance >= 15));
    }

    // Test: test_transfer_overflow_on_destination
    {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: u64::MAX };
        // Precondition dest.balance <= u64::MAX - amount is not met for amount = 1.
        // Thus, the transfer call is not permitted; we assert the violation.
        assert(!(dest.balance <= u64::MAX - 1));
    }
}

}