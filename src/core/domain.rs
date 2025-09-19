// Core business logic for the wallet application

pub struct Wallet {
    pub id: String,
    pub balance: u64,
}

impl Wallet {
    pub fn new(id: &str) -> Self {
        Wallet {
            id: id.to_string(),
            balance: 0,
        }
    }

    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<(), String> {
        if self.balance < amount {
            return Err("Insufficient balance".to_string());
        }
        self.balance -= amount;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new("test_wallet");
        assert_eq!(wallet.id, "test_wallet");
        assert_eq!(wallet.balance, 0);
    }

    #[test]
    fn test_wallet_deposit() {
        let mut wallet = Wallet::new("test_wallet");
        wallet.deposit(100);
        assert_eq!(wallet.balance, 100);
    }

    #[test]
    fn test_wallet_withdrawal_success() {
        let mut wallet = Wallet::new("test_wallet");
        wallet.deposit(100);
        let result = wallet.withdraw(50);
        assert!(result.is_ok());
        assert_eq!(wallet.balance, 50);
    }

    #[test]
    fn test_wallet_withdrawal_failure() {
        let mut wallet = Wallet::new("test_wallet");
        wallet.deposit(50);
        let result = wallet.withdraw(100);
        assert!(result.is_err());
        assert_eq!(wallet.balance, 50);
    }
}
