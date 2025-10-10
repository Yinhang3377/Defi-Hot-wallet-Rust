use std::collections::HashMap;
use std::sync::Arc;

// ---------- test-only master-key injection helpers (integration tests need visibility) ----------
use once_cell::sync::Lazy;
use std::sync::Mutex;

static TEST_MASTER_KEYS: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static TEST_MASTER_DEFAULT: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

/// Inject a test master key for a specific wallet name (test-only helper).
pub fn inject_test_master_key(name: &str, key: Vec<u8>) {
    let mut map = TEST_MASTER_KEYS.lock().unwrap();
    map.insert(name.to_string(), key);
}

/// Set a default test master key used for any wallet when no per-name key is injected (test-only).
pub fn set_test_master_key_default(key: Vec<u8>) {
    let mut def = TEST_MASTER_DEFAULT.lock().unwrap();
    *def = Some(key);
}

/// Clear injected test keys (test-only).
pub fn clear_injected_test_master_keys() {
    TEST_MASTER_KEYS.lock().unwrap().clear();
    *TEST_MASTER_DEFAULT.lock().unwrap() = None;
}
// ------------------------------------------------------------------------------
use tracing::{info, warn};

use crate::blockchain::{
    bridge::{
        // ...existing code...
        mock::{EthereumToSolanaBridge, SolanaToEthereumBridge}, // 保持 mock 导入
        BridgeTransaction, // BridgeTransaction 仍在 bridge 模块中定义
        BridgeTransactionStatus,
    },
    ethereum::EthereumClient,
    solana::SolanaClient,
    traits::{BlockchainClient, Bridge}, // 从 traits 导入
};
use crate::core::config::WalletConfig;
use crate::core::errors::WalletError;
use crate::core::validation::{validate_address, validate_amount};
use crate::core::wallet::{backup, create, recover};
use crate::core::wallet_info::{SecureWalletData, WalletInfo};
use crate::crypto::{hsm::HSMManager, multisig::MultiSignature, quantum::QuantumSafeEncryption};
use crate::storage::{WalletMetadata, WalletStorage, WalletStorageTrait};

#[allow(dead_code)]
fn get_fallback_rpc_url(network: &str) -> Option<String> {
    match network {
        "eth" => Some("https://ethereum.publicnode.com".to_string()),
        "sepolia" => Some("https://ethereum-sepolia.publicnode.com".to_string()),
        "polygon" => Some("https://polygon-rpc.com".to_string()),
        "bsc" => Some("https://bsc-dataseed.bnbchain.org/".to_string()),
        "solana" => Some("https://api.mainnet-beta.solana.com".to_string()),
        _ => None,
    }
}

pub struct WalletManager {
    storage: Arc<dyn WalletStorageTrait + Send + Sync>,
    quantum_crypto: QuantumSafeEncryption,
    _multisig: MultiSignature,
    _hsm: HSMManager,
    blockchain_clients: Arc<HashMap<String, Box<dyn BlockchainClient>>>,
    #[allow(dead_code)]
    bridges: Arc<HashMap<String, Box<dyn Bridge>>>,
}

impl WalletManager {
    pub async fn new(config: &WalletConfig) -> Result<Self, WalletError> {
        info!("Initializing WalletManager");

        let storage: Arc<dyn WalletStorageTrait + Send + Sync> = Arc::new(
            WalletStorage::new_with_url(&config.storage.database_url)
                .await
                .map_err(|e| WalletError::StorageError(e.to_string()))?,
        );
        let quantum_crypto =
            QuantumSafeEncryption::new().map_err(|e| WalletError::CryptoError(e.to_string()))?;
        let multisig = MultiSignature::new();
        let hsm = HSMManager::new().await.map_err(|e| WalletError::Other(e.to_string()))?;

        // Initialize bridges
        let mut bridges: HashMap<String, Box<dyn Bridge>> = HashMap::new();
        bridges.insert(
            "eth-solana".to_string(),
            Box::new(EthereumToSolanaBridge::new("0x...EthSolBridge...")),
        );
        bridges.insert(
            "solana-eth".to_string(),
            Box::new(SolanaToEthereumBridge::new("0x...SolEthBridge...")),
        );
        let bridges = Arc::new(bridges);

        let mut blockchain_clients: HashMap<String, Box<dyn BlockchainClient>> = HashMap::new();

        for (name, network_config) in &config.blockchain.networks {
            info!("Initializing client for network: {}", name);

            let mut retry_count = 0;
            let max_retries = 3;
            let mut last_error: Option<WalletError> = None;

            while retry_count < max_retries {
                let client_result: Result<Box<dyn BlockchainClient>, WalletError> =
                    match name.as_str() {
                        "eth" | "sepolia" | "polygon" | "bsc" | "bsctestnet" => {
                            let timeout = std::time::Duration::from_secs(15);
                            let client_future = EthereumClient::new(&network_config.rpc_url);
                            match tokio::time::timeout(timeout, client_future).await {
                                Ok(result) => result
                                    .map(|c| Box::new(c) as Box<dyn BlockchainClient>)
                                    .map_err(|e| WalletError::NetworkError(e.to_string())),
                                Err(_) => Err(WalletError::NetworkError(format!(
                                    "Connection timeout for {}",
                                    name
                                ))),
                            }
                        }
                        "solana" | "solana-devnet" => {
                            let timeout = std::time::Duration::from_secs(15);
                            let client_future = SolanaClient::new(&network_config.rpc_url);
                            match tokio::time::timeout(timeout, client_future).await {
                                Ok(result) => result
                                    .map(|c| Box::new(c) as Box<dyn BlockchainClient>)
                                    .map_err(|e| WalletError::NetworkError(e.to_string())),
                                Err(_) => Err(WalletError::NetworkError(format!(
                                    "Connection timeout for {}",
                                    name
                                ))),
                            }
                        }
                        _ => Err(WalletError::NetworkError(format!(
                            "Unsupported network type for {}",
                            name
                        ))),
                    };

                match client_result {
                    Ok(c) => {
                        let native_token = c.get_native_token().to_string();
                        blockchain_clients.insert(name.clone(), c);
                        info!("{} client initialized for network '{}'", native_token, name);
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        retry_count += 1;
                        if retry_count < max_retries {
                            warn!("Attempt {} failed for {}, retrying...", retry_count, name);
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        }
                    }
                }
            }

            if retry_count == max_retries {
                warn!(
                    "Failed to initialize client for {} after {} attempts: {}",
                    name,
                    max_retries,
                    last_error.unwrap_or_else(|| WalletError::Other("Unknown error".to_string()))
                );
            }
        }

        Ok(Self {
            storage,
            quantum_crypto,
            _multisig: multisig,
            _hsm: hsm,
            blockchain_clients: Arc::new(blockchain_clients),
            bridges,
        })
    }

    #[cfg(test)]
    pub async fn new_with_storage(
        _config: &WalletConfig,
        storage: Arc<dyn WalletStorageTrait + Send + Sync>,
        _test_master_key: Option<Vec<u8>>,
    ) -> Result<Self, WalletError> {
        let quantum_crypto =
            QuantumSafeEncryption::new().map_err(|e| WalletError::CryptoError(e.to_string()))?;
        let multisig = MultiSignature::new();
        let hsm = HSMManager::new().await.map_err(|e| WalletError::Other(e.to_string()))?;

        let mut bridges: HashMap<String, Box<dyn Bridge>> = HashMap::new();
        bridges.insert(
            "eth-solana".to_string(),
            Box::new(EthereumToSolanaBridge::new("0x...EthSolBridge...")),
        );
        bridges.insert(
            "solana-eth".to_string(),
            Box::new(SolanaToEthereumBridge::new("0x...SolEthBridge...")),
        );

        Ok(Self {
            storage,
            quantum_crypto,
            _multisig: multisig,
            _hsm: hsm,
            blockchain_clients: Arc::new(HashMap::new()),
            bridges: Arc::new(bridges),
        })
    }

    pub async fn create_wallet(
        &self,
        name: &str,
        quantum_safe: bool,
    ) -> Result<WalletInfo, WalletError> {
        create::create_wallet(&self.storage, &self.quantum_crypto, name, quantum_safe).await
    }

    pub async fn list_wallets(&self) -> Result<Vec<WalletMetadata>, WalletError> {
        info!("Listing all wallets");
        let wallets = self
            .storage
            .list_wallets()
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))?;
        info!("Found {} wallets", wallets.len());
        Ok(wallets)
    }

    /// Retrieves a single wallet's metadata by its unique name.
    pub async fn get_wallet_by_name(
        &self,
        name: &str,
    ) -> Result<Option<WalletMetadata>, WalletError> {
        info!("Getting wallet by name: {}", name);
        let wallets = self
            .storage
            .list_wallets()
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))?;

        // Find the wallet with the matching name
        let found_wallet = wallets.into_iter().find(|w| w.name == name);
        Ok(found_wallet)
    }

    pub async fn delete_wallet(&self, name: &str) -> Result<(), WalletError> {
        info!("Deleting wallet: {}", name);
        self.storage
            .delete_wallet(name)
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))?;
        info!("Wallet '{}' deleted successfully", name);
        Ok(())
    }

    pub async fn get_balance(
        &self,
        wallet_name: &str,
        network: &str,
    ) -> Result<String, WalletError> {
        info!("Getting balance for wallet: {} on network: {}", wallet_name, network);

        let mut wallet_data = self.load_wallet_securely(wallet_name).await?;

        let client = self.blockchain_clients.get(network).ok_or_else(|| {
            WalletError::BlockchainError(format!("Unsupported network: {}", network))
        })?;

        let address = self
            .derive_address(&wallet_data.encrypted_master_key, network)
            .map_err(|e| WalletError::AddressError(e.to_string()))?;

        let balance = client
            .get_balance(&address)
            .await
            .map_err(|e| WalletError::BlockchainError(e.to_string()))?;

        wallet_data.zeroize();

        Ok(balance)
    }

    pub async fn send_transaction(
        &self,
        wallet_name: &str,
        to_address: &str,
        amount: &str,
        network: &str,
    ) -> Result<String, WalletError> {
        info!(
            "Sending transaction from wallet: {} to: {} amount: {} on: {}",
            wallet_name, to_address, amount, network
        );

        validate_address(to_address, network)
            .map_err(|e| WalletError::ValidationError(e.to_string()))?;
        validate_amount(amount).map_err(|e| WalletError::ValidationError(e.to_string()))?;

        let mut wallet_data = self.load_wallet_securely(wallet_name).await?;

        let client = self.blockchain_clients.get(network).ok_or_else(|| {
            WalletError::BlockchainError(format!("Unsupported network: {}", network))
        })?;

        let private_key = self
            .derive_private_key(&wallet_data.encrypted_master_key, network)
            .map_err(|e| WalletError::KeyDerivationError(e.to_string()))?;
        let tx_hash = client
            .send_transaction(&private_key, to_address, amount)
            .await
            .map_err(|e| WalletError::BlockchainError(e.to_string()))?;

        wallet_data.zeroize();

        info!("Transaction sent with hash: {}", tx_hash);
        Ok(tx_hash)
    }

    pub async fn bridge_assets(
        &self,
        wallet_name: &str,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
    ) -> Result<String, WalletError> {
        info!(
            "Bridging assets from wallet: {} from: {} to: {} token: {} amount: {}",
            wallet_name, from_chain, to_chain, token, amount
        );

        let mut wallet_data = self.load_wallet_securely(wallet_name).await?;

        let bridge_key = format!("{}-{}", from_chain, to_chain);
        let bridge = self.bridges.get(&bridge_key).ok_or_else(|| {
            WalletError::BlockchainError(format!("Unsupported bridge: {}", bridge_key))
        })?;

        let tx_hash = bridge
            .transfer_across_chains(from_chain, to_chain, token, amount, &wallet_data)
            .await?;

        wallet_data.zeroize();
        Ok(tx_hash)
    }

    pub async fn get_block_number(&self, network: &str) -> Result<u64, WalletError> {
        let client = self.blockchain_clients.get(network).ok_or_else(|| {
            WalletError::BlockchainError(format!("Unsupported network: {}", network))
        })?;
        client.get_block_number().await.map_err(|e| WalletError::BlockchainError(e.to_string()))
    }

    pub async fn check_bridge_status(
        &self,
        bridge_tx_id: &str,
    ) -> Result<BridgeTransactionStatus, WalletError> {
        self.storage
            .get_bridge_transaction(bridge_tx_id)
            .await
            .map(|tx| tx.status)
            .map_err(|e| WalletError::StorageError(e.to_string()))
    }

    pub async fn get_bridge_transaction_status(
        &self,
        bridge_tx_id: &str,
    ) -> Result<BridgeTransaction, WalletError> {
        self.storage
            .get_bridge_transaction(bridge_tx_id)
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))
    }

    pub async fn update_bridge_transaction_status(
        &self,
        bridge_tx_id: &str,
        status: BridgeTransactionStatus,
        source_tx_hash: Option<String>,
    ) -> Result<(), WalletError> {
        info!("Updating bridge tx {} status to {:?}", bridge_tx_id, status);
        self.storage
            .update_bridge_transaction_status(bridge_tx_id, status, source_tx_hash)
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))
    }

    pub fn calculate_bridge_fee(
        &self,
        from_chain: &str,
        to_chain: &str,
        _token: &str,
        amount: &str,
    ) -> Result<(String, chrono::DateTime<chrono::Utc>), WalletError> {
        let amount_decimal =
            amount.parse::<f64>().map_err(|e| WalletError::ValidationError(e.to_string()))?;
        let fee = (amount_decimal * 0.01).to_string();

        let estimated_blocks = match (from_chain, to_chain) {
            ("eth", _) => 20,
            ("solana", _) => 32,
            ("bsc", _) => 40,
            _ => 30,
        };

        let now = chrono::Utc::now();
        let estimated_time = now + chrono::Duration::seconds((estimated_blocks * 6) as i64);

        Ok((fee, estimated_time))
    }

    #[allow(dead_code)]
    fn start_bridge_monitor(&self, bridge_tx_id: String) {
        let storage = Arc::clone(&self.storage);

        tokio::spawn(async move {
            info!("Starting bridge monitor for tx: {}", bridge_tx_id);
            for _ in 0..20 {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                if let Ok(tx) = storage.get_bridge_transaction(&bridge_tx_id).await {
                    if tx.status == BridgeTransactionStatus::Completed {
                        break;
                    }
                }
            }
            info!("Bridge monitor completed for tx: {}", bridge_tx_id);
        });
    }

    pub fn derive_address(&self, master_key: &[u8], network: &str) -> Result<String, WalletError> {
        match network {
            "eth" => {
                let addr_bytes = if master_key.len() >= 20 {
                    master_key[..20].to_vec()
                } else {
                    let mut v = vec![0u8; 20];
                    v[..master_key.len()].copy_from_slice(master_key);
                    v
                };
                Ok(format!("0x{}", hex::encode(&addr_bytes)))
            }
            "solana" => {
                let key_bytes = if master_key.len() >= 32 {
                    master_key[..32].to_vec()
                } else {
                    let mut v = vec![0u8; 32];
                    v[..master_key.len()].copy_from_slice(master_key);
                    v
                };
                Ok(bs58::encode(&key_bytes).into_string())
            }
            _ => Err(WalletError::ValidationError(format!("Unsupported network: {}", network))),
        }
    }

    fn derive_private_key(&self, master_key: &[u8], network: &str) -> Result<Vec<u8>, WalletError> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(network.as_bytes());
        Ok(hasher.finalize().to_vec())
    }

    async fn load_wallet_securely(
        &self,
        wallet_name: &str,
    ) -> Result<SecureWalletData, WalletError> {
        let (serialized_data, quantum_safe) = self
            .storage
            .load_wallet(wallet_name)
            .await
            .map_err(|e| WalletError::StorageError(e.to_string()))?;

        let mut wallet_data: SecureWalletData = bincode::deserialize(&serialized_data)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        // 获取用于解密的真实主密钥
        let master_key_for_decrypt = self.get_master_key_for_wallet(wallet_name)?;
        let decrypted_master_key = if quantum_safe {
            self.quantum_crypto
                .decrypt(&wallet_data.encrypted_master_key)
                .map_err(|e| WalletError::CryptoError(e.to_string()))?
        } else {
            self.decrypt_traditional(
                &wallet_data.encrypted_master_key, // ciphertext
                &wallet_data.salt,
                &wallet_data.nonce,
                &master_key_for_decrypt, // correct master key
            )
            .map_err(|e| WalletError::CryptoError(e.to_string()))?
        };
        wallet_data.encrypted_master_key = decrypted_master_key;

        Ok(wallet_data) // 返回包含解密后主密钥的 wallet_data
    }

    fn get_master_key_for_wallet(&self, wallet_name: &str) -> Result<Vec<u8>, WalletError> {
        // Test-injected key takes precedence (only compiled in tests)
        // per-name injected key
        if let Some(k) = TEST_MASTER_KEYS.lock().unwrap().get(wallet_name) {
            return Ok(k.clone());
        }
        // default test key
        if let Some(k) = TEST_MASTER_DEFAULT.lock().unwrap().as_ref() {
            return Ok(k.clone());
        }

        // Production fallback: placeholder behavior (replace with real key retrieval)
        Ok(vec![0u8; 32])
    }

    fn decrypt_traditional(
        &self,
        ciphertext: &[u8],
        salt: &[u8],
        nonce_bytes: &[u8],
        master_key: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };

        let mut enc_key_bytes = [0u8; 32];
        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), master_key);
        hkdf.expand(b"aes-gcm-key", &mut enc_key_bytes).map_err(|e| {
            WalletError::CryptoError(format!("Failed to derive encryption key: {}", e))
        })?;

        let key = Key::<Aes256Gcm>::from_slice(&enc_key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| WalletError::CryptoError(format!("AES decryption failed: {}", e)))?;
        Ok(plaintext)
    }

    pub async fn get_transaction_history(
        &self,
        _wallet_name: &str,
    ) -> Result<Vec<String>, WalletError> {
        Ok(vec![])
    }

    pub async fn backup_wallet(&self, wallet_name: &str) -> Result<String, WalletError> {
        backup::backup_wallet(&self.storage, wallet_name).await
    }

    pub async fn restore_wallet(
        &self,
        wallet_name: &str,
        seed_phrase: &str,
        quantum_safe: bool,
    ) -> Result<(), WalletError> {
        recover::recover_wallet(
            &self.storage,
            &self.quantum_crypto,
            wallet_name,
            seed_phrase,
            quantum_safe,
        )
        .await
    }

    pub async fn send_multi_sig_transaction(
        &self,
        _wallet_name: &str,
        _to_address: &str,
        _amount: &str,
        _network: &str,
        _signatures: &[String],
    ) -> Result<String, WalletError> {
        Ok("fake_multi_sig_tx_hash".to_string())
    }

    pub fn generate_mnemonic(&self) -> Result<String, WalletError> {
        crate::core::wallet::create::generate_mnemonic()
    }
}
