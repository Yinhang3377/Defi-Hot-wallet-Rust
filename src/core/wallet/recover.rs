// filepath: src/core/wallet/recover.rs
use anyhow::Result;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
#[allow(unused_imports)]
use zeroize::Zeroize; // 允许未使用的导入，修复编译器误报

use crate::core::errors::WalletError;
use crate::core::wallet_info::{SecureWalletData, WalletInfo}; // Assuming this is correct
use crate::storage::WalletStorageTrait;

/// (ciphertext, salt, nonce)
type WalletKeyMaterial = (Vec<u8>, Vec<u8>, Vec<u8>);

pub async fn recover_wallet(
    storage: &Arc<dyn WalletStorageTrait + Send + Sync>,
    quantum_crypto: &crate::crypto::quantum::QuantumSafeEncryption,
    wallet_name: &str,
    seed_phrase: &str,
    quantum_safe: bool,
) -> Result<(), WalletError> {
    info!("Recovering wallet: {} from seed phrase", wallet_name);

    let wallets =
        storage.list_wallets().await.map_err(|e| WalletError::StorageError(e.to_string()))?;
    if wallets.iter().any(|w| w.name == wallet_name) {
        return Err(WalletError::StorageError(format!("Wallet already exists: {}", wallet_name)));
    }

    let master_key_vec = derive_master_key(seed_phrase)
        .await
        .map_err(|e| WalletError::KeyDerivationError(e.to_string()))?;
    let mut master_key = [0u8; 32];
    if master_key_vec.len() >= 32 {
        master_key.copy_from_slice(&master_key_vec[..32]);
    } else {
        let mut tmp = [0u8; 32];
        tmp[..master_key_vec.len()].copy_from_slice(&master_key_vec);
        master_key.copy_from_slice(&tmp);
    }

    let wallet_info = WalletInfo {
        id: Uuid::new_v4(),
        name: wallet_name.to_string(),
        created_at: chrono::Utc::now(),
        quantum_safe,
        multi_sig_threshold: 2,
        networks: vec!["eth".to_string(), "solana".to_string()],
    };

    let mut encrypted_wallet_data = SecureWalletData {
        info: wallet_info.clone(),
        encrypted_master_key: Vec::new(),
        salt: Vec::new(),
        nonce: Vec::new(),
    };

    store_wallet_securely(
        storage,
        quantum_crypto,
        &mut encrypted_wallet_data,
        &master_key,
        quantum_safe,
    )
    .await?;
    encrypted_wallet_data.zeroize();

    Ok(())
}

async fn derive_master_key(mnemonic: &str) -> Result<Vec<u8>, WalletError> {
    use bip39::{Language, Mnemonic};

    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|e| WalletError::MnemonicError(format!("Failed to parse mnemonic: {}", e)))?;
    let seed_bytes = mnemonic.to_seed("");
    Ok(seed_bytes[..32].to_vec())
}

async fn store_wallet_securely(
    storage: &Arc<dyn WalletStorageTrait + Send + Sync>,
    quantum_crypto: &crate::crypto::quantum::QuantumSafeEncryption,
    wallet_data: &mut SecureWalletData,
    master_key: &[u8; 32],
    quantum_safe: bool,
) -> Result<(), WalletError> {
    let (encrypted_key, salt, nonce) = if quantum_safe {
        let encrypted = quantum_crypto
            .encrypt(master_key)
            .map_err(|e| WalletError::CryptoError(e.to_string()))?;
        (encrypted, vec![], vec![])
    } else {
        encrypt_traditional(master_key, master_key)
            .map_err(|e| WalletError::CryptoError(e.to_string()))?
    };

    wallet_data.encrypted_master_key = encrypted_key;
    wallet_data.salt = salt;
    wallet_data.nonce = nonce;

    let serialized_data = bincode::serialize(wallet_data)
        .map_err(|e| WalletError::SerializationError(e.to_string()))?;

    storage
        .store_wallet(&wallet_data.info.name, &serialized_data, quantum_safe)
        .await
        .map_err(|e| WalletError::StorageError(e.to_string()))?;
    Ok(())
}

fn encrypt_traditional(data: &[u8], master_key: &[u8]) -> Result<WalletKeyMaterial, WalletError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Key, Nonce,
    };
    use hkdf::Hkdf;
    use rand::RngCore;
    use sha2::Sha256;

    let mut enc_key_bytes = [0u8; 32];
    let hkdf = Hkdf::<Sha256>::new(Some(b"enc-salt"), master_key);
    hkdf.expand(b"aes-gcm-key", &mut enc_key_bytes)
        .map_err(|e| WalletError::CryptoError(format!("Failed to derive key: {}", e)))?;

    let key = Key::<Aes256Gcm>::from_slice(&enc_key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| WalletError::CryptoError(format!("AES encrypt failed: {}", e)))?;
    Ok((ciphertext, b"enc-salt".to_vec(), nonce_bytes.to_vec()))
}
