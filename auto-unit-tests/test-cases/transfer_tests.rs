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
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 30);
        assert_eq!(orig.balance, 70);
        assert_eq!(dest.balance, 80);
    }

    #[test]
    fn test_zero_transfer() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 50 };
        transfer(&mut orig, &mut dest, 0);
        assert_eq!(orig.balance, 100);
        assert_eq!(dest.balance, 50);
    }

    #[test]
    fn test_full_balance_transfer() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 25 };
        transfer(&mut orig, &mut dest, 100);
        assert_eq!(orig.balance, 0);
        assert_eq!(dest.balance, 125);
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds_should_panic() {
        // In debug mode, subtracting a larger number than available will panic.
        let mut orig = Account { balance: 50 };
        let mut dest = Account { balance: 100 };
        // This should trigger a panic due to underflow on orig.balance
        transfer(&mut orig, &mut dest, 60);
    }

    #[test]
    fn test_transfer_edge_values() {
        // Test transferring with amount equal to 0 and maximum value edge cases
        // For max boundary, we need to test that adding doesn't cause wrapping in debug mode.
        // However, since Account.balance is u64 and transfer does unchecked addition,
        // this test is only meaningful in a controlled scenario.
        
        // Setup for a transfer that doesn't panic.
        let mut orig = Account { balance: u64::MAX - 10 };
        let mut dest = Account { balance: 10 };
        transfer(&mut orig, &mut dest, 10);
        assert_eq!(orig.balance, u64::MAX - 20);
        assert_eq!(dest.balance, 20);
    }
}