// Application workflows for the wallet application
use crate::core::domain::Wallet;

pub struct WalletService {
    pub wallet: Wallet,
}

impl WalletService {
    pub fn new(wallet_id: &str) -> Self {
        WalletService {
            wallet: Wallet::new(wallet_id),
        }
    }

    pub fn process_deposit(&mut self, amount: u64) {
        self.wallet.deposit(amount);
    }

    pub fn process_withdrawal(&mut self, amount: u64) -> Result<(), String> {
        self.wallet.withdraw(amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_service_creation() {
        let service = WalletService::new("test_wallet");
        assert_eq!(service.wallet.id, "test_wallet");
        assert_eq!(service.wallet.balance, 0);
    }

    #[test]
    fn test_wallet_service_deposit() {
        let mut service = WalletService::new("test_wallet");
        service.process_deposit(100);
        assert_eq!(service.wallet.balance, 100);
    }

    #[test]
    fn test_wallet_service_withdrawal_success() {
        let mut service = WalletService::new("test_wallet");
        service.process_deposit(100);
        let result = service.process_withdrawal(50);
        assert!(result.is_ok());
        assert_eq!(service.wallet.balance, 50);
    }

    #[test]
    fn test_wallet_service_withdrawal_failure() {
        let mut service = WalletService::new("test_wallet");
        service.process_deposit(50);
        let result = service.process_withdrawal(100);
        assert!(result.is_err());
        assert_eq!(service.wallet.balance, 50);
    }
}
