pub struct Account {
    pub balance: u64,
}

pub fn transfer(orig: &mut Account, dest: &mut Account, amount: u64) {
    orig.balance = orig.balance - amount;
    dest.balance = dest.balance + amount;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_valid() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 20);
        assert_eq!(orig.balance, 80);
        assert_eq!(dest.balance, 70);
    }

    #[test]
    fn test_transfer_zero() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 0);
        assert_eq!(orig.balance, 100);
        assert_eq!(dest.balance, 50);
    }

    #[test]
    #[should_panic]
    fn test_transfer_insufficient_funds() {
        let mut orig = Account { balance: 10 };
        let mut dest = Account { balance: 20 };
        // This should panic in debug mode due to underflow in subtraction.
        transfer(&mut orig, &mut dest, 15);
    }

    #[test]
    fn test_transfer_full_balance() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 200 };
        transfer(&mut orig, &mut dest, 100);
        assert_eq!(orig.balance, 0);
        assert_eq!(dest.balance, 300);
    }

    #[test]
    #[should_panic]
    fn test_transfer_overflow_on_destination() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: u64::MAX };
        // This transfer should panic in debug mode due to overflow in addition.
        transfer(&mut orig, &mut dest, 1);
    }
}