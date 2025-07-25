fn main() {}

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
        let mut orig = Account { balance: 1000 };
        let mut dest = Account { balance: 500 };
        transfer(&mut orig, &mut dest, 200);
        assert_eq!(orig.balance, 800);
        assert_eq!(dest.balance, 700);
    }

    #[test]
    fn test_transfer_zero() {
        let mut orig = Account { balance: 1000 };
        let mut dest = Account { balance: 500 };
        transfer(&mut orig, &mut dest, 0);
        assert_eq!(orig.balance, 1000);
        assert_eq!(dest.balance, 500);
    }

    #[test]
    #[should_panic]
    fn test_insufficient_funds() {
        let mut orig = Account { balance: 100 };
        let mut dest = Account { balance: 500 };
        // This should panic in debug mode due to subtraction underflow.
        transfer(&mut orig, &mut dest, 200);
    }

    #[test]
    #[should_panic]
    fn test_destination_overflow() {
        // Set up such that dest.balance + amount overflows u64.
        // For example, if dest.balance is u64::MAX - 5 and we transfer 10, then addition will overflow.
        let mut orig = Account { balance: 10 };
        let mut dest = Account { balance: u64::MAX - 5 };
        transfer(&mut orig, &mut dest, 10);
    }

    #[test]
    fn test_transfer_entire_balance() {
        let mut orig = Account { balance: 500 };
        let mut dest = Account { balance: 300 };
        transfer(&mut orig, &mut dest, 500);
        assert_eq!(orig.balance, 0);
        assert_eq!(dest.balance, 800);
    }
}