// Infrastructure layer for managing external dependencies

use std::collections::HashMap;
use std::sync::Mutex;

pub struct WalletRepository {
    pub storage: Mutex<HashMap<String, u64>>,
}

impl WalletRepository {
    pub fn new() -> Self {
        WalletRepository {
            storage: Mutex::new(HashMap::new()),
        }
    }

    pub fn save(&self, wallet_id: &str, balance: u64) {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(wallet_id.to_string(), balance);
    }

    pub fn load(&self, wallet_id: &str) -> Option<u64> {
        let storage = self.storage.lock().unwrap();
        storage.get(wallet_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_repository_save_and_load() {
        let repo = WalletRepository::new();
        repo.save("test_wallet", 100);
        let balance = repo.load("test_wallet");
        assert_eq!(balance, Some(100));
    }

    #[test]
    fn test_wallet_repository_load_nonexistent() {
        let repo = WalletRepository::new();
        let balance = repo.load("nonexistent_wallet");
        assert_eq!(balance, None);
    }
}
