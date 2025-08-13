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
        let mut account_a = Account { balance: 100 };
        let mut account_b = Account { balance: 50 };
        transfer(&mut account_a, &mut account_b, 20);
        assert_eq!(account_a.balance, 80);
        assert_eq!(account_b.balance, 70);
    }

    #[test]
    fn test_zero_transfer() {
        let mut account_a = Account { balance: 100 };
        let mut account_b = Account { balance: 200 };
        transfer(&mut account_a, &mut account_b, 0);
        assert_eq!(account_a.balance, 100);
        assert_eq!(account_b.balance, 200);
    }

    #[test]
    fn test_full_amount_transfer() {
        let mut account_a = Account { balance: 100 };
        let mut account_b = Account { balance: 0 };
        transfer(&mut account_a, &mut account_b, 100);
        assert_eq!(account_a.balance, 0);
        assert_eq!(account_b.balance, 100);
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds() {
        let mut account_a = Account { balance: 50 };
        let mut account_b = Account { balance: 0 };
        // This should panic due to underflow when subtracting more than available.
        transfer(&mut account_a, &mut account_b, 60);
    }

    #[test]
    fn test_large_values() {
        let mut account_a = Account { balance: u64::MAX };
        let mut account_b = Account { balance: 0 };
        transfer(&mut account_a, &mut account_b, 1);
        assert_eq!(account_a.balance, u64::MAX - 1);
        assert_eq!(account_b.balance, 1);
    }

    #[test]
    #[should_panic]
    fn test_overflow_on_destination() {
        let mut account_a = Account { balance: 100 };
        let mut account_b = Account { balance: u64::MAX };
        // This should panic due to overflow on addition in the destination account.
        transfer(&mut account_a, &mut account_b, 1);
    }
}