pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) {
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
}

pub fn test(init_balance: u64, transfer_amount: u64) {
    let mut acc1 = Account { balance: init_balance };
    let mut acc2 = Account { balance: 0 };
    transfer(&mut acc1, &mut acc2, transfer_amount);
    assert!(acc1.balance == init_balance - transfer_amount);
    assert!(acc2.balance == transfer_amount);
}

pub fn main() {
}