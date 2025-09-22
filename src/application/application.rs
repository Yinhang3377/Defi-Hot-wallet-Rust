use crate::audit::logging::log_operation;
use crate::core::domain::{Tx, Wallet};
use crate::crypto::kdf::Encryptor;

pub struct WalletService {
    #[allow(dead_code)]
    enc: Encryptor,
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            enc: Encryptor::new(),
        }
    }

    pub fn with_encryptor(enc: Encryptor) -> Self {
        Self { enc }
    }

    pub async fn create_wallet(&self, mnemonic: &str) -> anyhow::Result<Wallet> {
        let w = Wallet::from_mnemonic(mnemonic)?;
        log_operation("create_wallet", &w.id, true);
        Ok(w)
    }
    pub async fn send_tx(&self, w: &Wallet, to: &str, amount: u64) -> anyhow::Result<Tx> {
        let tx = Tx::new(w, to, amount);
        log_operation("send_tx", &w.id, true);
        Ok(tx)
    }
}

impl Default for WalletService {
    fn default() -> Self {
        Self::new()
    }
}
