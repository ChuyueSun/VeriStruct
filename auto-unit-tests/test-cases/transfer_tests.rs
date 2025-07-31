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
    fn test_normal_transfer() {
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: 500 };
        transfer(&mut origin, &mut destination, 300);
        assert_eq!(origin.balance, 700);
        assert_eq!(destination.balance, 800);
    }

    #[test]
    fn test_transfer_zero_amount() {
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: 500 };
        transfer(&mut origin, &mut destination, 0);
        assert_eq!(origin.balance, 1000);
        assert_eq!(destination.balance, 500);
    }

    #[test]
    fn test_transfer_full_balance() {
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: 500 };
        transfer(&mut origin, &mut destination, 1000);
        assert_eq!(origin.balance, 0);
        assert_eq!(destination.balance, 1500);
    }

    #[test]
    #[should_panic]
    fn test_transfer_insufficient_funds() {
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: 500 };
        // This should panic because origin.balance < amount, triggering an underflow.
        transfer(&mut origin, &mut destination, 1500);
    }

    #[test]
    fn test_transfer_no_overflow_in_destination() {
        // Test transferring an amount that causes the destination balance to exactly equal u64::MAX.
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: u64::MAX - 500 };
        transfer(&mut origin, &mut destination, 500);
        assert_eq!(origin.balance, 500);
        assert_eq!(destination.balance, u64::MAX);
    }

    #[test]
    #[should_panic]
    fn test_transfer_causes_destination_overflow() {
        // This should panic because the destination balance will overflow.
        let mut origin = Account { balance: 1000 };
        let mut destination = Account { balance: u64::MAX - 100 };
        transfer(&mut origin, &mut destination, 200);
    }
}