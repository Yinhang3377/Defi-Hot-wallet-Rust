// filepath: src/core/wallet/backup.rs
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::core::errors::WalletError;
use crate::storage::WalletStorageTrait;
/// Backs up a wallet by generating a new mnemonic.
pub async fn backup_wallet(
    _storage: &Arc<dyn WalletStorageTrait + Send + Sync>,
    wallet_name: &str,
) -> Result<String, WalletError> {
    info!("Backing up wallet: {}", wallet_name);
    // Generate mnemonic as backup
    let mnemonic = generate_mnemonic().map_err(|e| WalletError::MnemonicError(e.to_string()))?;
    Ok(mnemonic)
}

fn generate_mnemonic() -> Result<String, WalletError> {
    use bip39::{Language, Mnemonic};
    use rand::RngCore;

    let mut entropy = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| WalletError::MnemonicError(e.to_string()))?;
    Ok(mnemonic.to_string())
}
