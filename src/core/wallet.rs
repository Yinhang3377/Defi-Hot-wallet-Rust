use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::blockchain::{ethereum::EthereumClient, solana::SolanaClient, traits::BlockchainClient};
use crate::core::config::WalletConfig;
use crate::crypto::{
    hsm::HSMManager, multisig::MultiSignature, quantum::QuantumSafeEncryption, shamir,
};
use crate::storage::{WalletMetadata, WalletStorage};

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub quantum_safe: bool,
    pub multi_sig_threshold: u8,
    pub networks: Vec<String>,
}

// 为 WalletInfo 实现 Serialize，以便在 SecureWalletData 中使用
impl Serialize for WalletInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WalletInfo", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("quantum_safe", &self.quantum_safe)?;
        state.serialize_field("multi_sig_threshold", &self.multi_sig_threshold)?;
        state.serialize_field("networks", &self.networks)?;
        state.end()
    }
}

// 为 WalletInfo 添加 Deserialize 实现
impl<'de> Deserialize<'de> for WalletInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct WalletInfoVisitor;

        impl<'de> Visitor<'de> for WalletInfoVisitor {
            type Value = WalletInfo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WalletInfo")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WalletInfo, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut created_at = None;
                let mut quantum_safe = None;
                let mut multi_sig_threshold = None;
                let mut networks = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        "created_at" => created_at = Some(map.next_value()?),
                        "quantum_safe" => quantum_safe = Some(map.next_value()?),
                        "multi_sig_threshold" => multi_sig_threshold = Some(map.next_value()?),
                        "networks" => networks = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let created_at = created_at.ok_or_else(|| de::Error::missing_field("created_at"))?;
                let quantum_safe =
                    quantum_safe.ok_or_else(|| de::Error::missing_field("quantum_safe"))?;
                let multi_sig_threshold = multi_sig_threshold
                    .ok_or_else(|| de::Error::missing_field("multi_sig_threshold"))?;
                let networks = networks.ok_or_else(|| de::Error::missing_field("networks"))?;

                Ok(WalletInfo {
                    id,
                    name,
                    created_at,
                    quantum_safe,
                    multi_sig_threshold,
                    networks,
                })
            }
        }

        deserializer.deserialize_struct("WalletInfo", &["id", "name", "created_at", "quantum_safe", "multi_sig_threshold", "networks"], WalletInfoVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecureWalletData {
    #[zeroize(skip)]
    pub info: WalletInfo,
    pub encrypted_master_key: Vec<u8>,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub struct WalletManager {
    storage: WalletStorage,
    quantum_crypto: QuantumSafeEncryption,
    _multisig: MultiSignature,
    _hsm: HSMManager,
    blockchain_clients: Arc<HashMap<String, Box<dyn BlockchainClient>>>,
}

impl WalletManager {
    pub async fn new(config: &WalletConfig) -> Result<Self> {
        info!("🔧 Initializing WalletManager");

        let storage = WalletStorage::new().await?;
        let quantum_crypto = QuantumSafeEncryption::new()?;
        let multisig = MultiSignature::new();
        let hsm = HSMManager::new().await?;

        let mut blockchain_clients: HashMap<String, Box<dyn BlockchainClient>> = HashMap::new();

        for (name, network_config) in &config.blockchain.networks {
            info!("Initializing client for network: {}", name);
            let client: Result<Box<dyn BlockchainClient>> = match name.as_str() {
                "eth" | "sepolia" | "polygon" | "bsc" | "bsctestnet" => {
                    EthereumClient::new(&network_config.rpc_url)
                        .await
                        .map(|c| Box::new(c) as Box<dyn BlockchainClient>)
                }
                "solana" | "solana-devnet" => SolanaClient::new(&network_config.rpc_url)
                    .await
                    .map(|c| Box::new(c) as Box<dyn BlockchainClient>),
                _ => Err(anyhow::anyhow!("Unsupported network type for {}", name)),
            };
            match client {
                Ok(c) => {
                    // 先获取日志所需信息，避免在移动 c 之后再借用它
                    let native_token = c.get_native_token().to_string();
                    blockchain_clients.insert(name.clone(), c);
                    info!(
                        "✅ {} client initialized for network '{}'",
                        native_token, name
                    );
                }
                Err(e) => warn!("⚠️ Failed to initialize client for {}: {}", name, e),
            }
        }

        Ok(Self {
            storage,
            quantum_crypto,
            _multisig: multisig,
            _hsm: hsm,
            blockchain_clients: Arc::new(blockchain_clients),
        })
    }

    pub async fn create_wallet(&self, name: &str, quantum_safe: bool) -> Result<WalletInfo> {
        info!(
            "🔐 Creating new wallet: {} (quantum_safe: {})",
            name, quantum_safe
        );

        // Generate mnemonic phrase
        let mnemonic = self.generate_mnemonic()?;

        // Generate master key from mnemonic
        let master_key_vec = self.derive_master_key(&mnemonic)?;
        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&master_key_vec);
        drop(master_key_vec); // 立即释放包含完整种子的 Vec

        // Create wallet info
        let wallet_info = WalletInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            quantum_safe,
            multi_sig_threshold: 2,
            networks: vec!["eth".to_string(), "solana".to_string()],
        };

        // Create Shamir secret shares (2-of-3 threshold)
        let shamir_shares_tuples = crate::crypto::shamir::split_secret(master_key, 2, 3)?;
        let shamir_shares: Vec<Vec<u8>> = shamir_shares_tuples
            .into_iter()
            .map(|(id, bytes)| {
                let mut share = Vec::with_capacity(33); // 1-byte ID + 32-byte data
                share.push(id);
                share.extend_from_slice(&bytes);
                share
            })
            .collect();

        // Create secure wallet data
        let mut encrypted_wallet_data = SecureWalletData {
            info: wallet_info.clone(),
            encrypted_master_key: Vec::new(), // Placeholder
            salt: Vec::new(),                 // Placeholder
            nonce: Vec::new(),                // Placeholder
        };

        // Encrypt and store wallet
        self.store_wallet_securely(&mut encrypted_wallet_data, &master_key, quantum_safe)
            .await?;

        // Clear sensitive data from memory
        encrypted_wallet_data.zeroize();

        info!("✅ Wallet '{}' created with ID: {}", name, wallet_info.id);
        Ok(wallet_info)
    }

    pub async fn list_wallets(&self) -> Result<Vec<WalletMetadata>> {
        info!("Listing all wallets");
        let wallets = self.storage.list_wallets().await?;
        info!("Found {} wallets", wallets.len());
        Ok(wallets)
    }

    pub async fn delete_wallet(&self, name: &str) -> Result<()> {
        info!("Deleting wallet: {}", name);
        self.storage.delete_wallet(name).await?;
        info!("✅ Wallet '{}' deleted successfully", name);
        Ok(())
    }

    pub async fn get_balance(&self, wallet_name: &str, network: &str) -> Result<String> {
        info!(
            "💰 Getting balance for wallet: {} on network: {}",
            wallet_name, network
        );

        // Load wallet
        let mut wallet_data = self.load_wallet_securely(wallet_name).await?;

        // Get blockchain client
        let client = self
            .blockchain_clients
            .get(network)
            .ok_or_else(|| anyhow::anyhow!("Unsupported network: {}", network))?;

        // Derive address for the network
        let address = self.derive_address(&wallet_data.encrypted_master_key, network)?;

        // Get balance from blockchain
        let balance = client.get_balance(&address).await?;

        // Zeroize sensitive data after use
        wallet_data.zeroize();

        Ok(balance)
    }

    pub async fn send_transaction(
        &self,
        wallet_name: &str,
        to_address: &str,
        amount: &str,
        network: &str,
    ) -> Result<String> {
        info!(
            "💸 Sending transaction from wallet: {} to: {} amount: {} on: {}",
            wallet_name, to_address, amount, network
        );

        // Load wallet
        let mut wallet_data = self.load_wallet_securely(wallet_name).await?;

        // Get blockchain client
        let client = self
            .blockchain_clients
            .get(network)
            .ok_or_else(|| anyhow::anyhow!("Unsupported network: {}", network))?;

        // Create and sign transaction
        let private_key = self.derive_private_key(&wallet_data.encrypted_master_key, network)?;
        let tx_hash = client
            .send_transaction(&private_key, to_address, amount)
            .await?;

        // Zeroize sensitive data after use
        wallet_data.zeroize();

        info!("✅ Transaction sent with hash: {}", tx_hash);
        Ok(tx_hash)
    }

    fn generate_mnemonic(&self) -> Result<String> {
        use bip39::{Language, Mnemonic};
        use rand::RngCore;

        let mut entropy = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        Ok(mnemonic.to_string())
    }

    /// Derives a 32-byte master key from a mnemonic phrase according to BIP39.
    /// It generates a 64-byte seed and returns the first 32 bytes, which is a common practice for BIP32.
    fn derive_master_key(&self, mnemonic: &str) -> Result<Vec<u8>> {
        use bip39::{Language, Mnemonic};

        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)?;
        // to_seed generates a 64-byte seed.
        let seed_bytes = mnemonic.to_seed("");
        // We use the first 32 bytes as the master key.
        Ok(seed_bytes[..32].to_vec())
    }

    fn derive_address(&self, master_key: &[u8], network: &str) -> Result<String> {
        // Implementation would derive network-specific addresses
        // This is a simplified version
        match network {
            "eth" => {
                // Derive Ethereum address using BIP44 path m/44'/60'/0'/0/0
                Ok(format!("0x{}", hex::encode(&master_key[..20])))
            }
            "solana" => {
                // Derive Solana address using bs58
                Ok(bs58::encode(&master_key[..32]).into_string())
            }
            _ => Err(anyhow::anyhow!("Unsupported network: {}", network)),
        }
    }

    fn derive_private_key(&self, master_key: &[u8], network: &str) -> Result<Vec<u8>> {
        // Simplified private key derivation
        // In production, this would use proper BIP32/BIP44 derivation
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        hasher.update(network.as_bytes());
        Ok(hasher.finalize().to_vec())
    }

    async fn store_wallet_securely(
        &self,
        wallet_data: &mut SecureWalletData,
        master_key: &[u8; 32],
        quantum_safe: bool,
    ) -> Result<()> {
        let (encrypted_key, salt, nonce) = if quantum_safe {
            let encrypted = self.quantum_crypto.encrypt(master_key)?;
            // For quantum, salt/nonce are part of the ciphertext format
            (encrypted, vec![], vec![])
        } else {
            // Use traditional AES-GCM encryption as fallback
            self.encrypt_traditional(master_key, master_key)?
        };

        wallet_data.encrypted_master_key = encrypted_key;
        wallet_data.salt = salt;
        wallet_data.nonce = nonce;

        let serialized_data = bincode::serialize(wallet_data)?;

        self.storage
            .store_wallet(&wallet_data.info.name, &serialized_data, quantum_safe)
            .await?;
        Ok(())
    }

    async fn load_wallet_securely(&self, wallet_name: &str) -> Result<SecureWalletData> {
        let (serialized_data, quantum_safe) = self.storage.load_wallet(wallet_name).await?;

        let mut wallet_data: SecureWalletData = bincode::deserialize(&serialized_data)?;

        let master_key = if quantum_safe {
            self.quantum_crypto.decrypt(&wallet_data.encrypted_master_key)?
        } else {
            self.decrypt_traditional(&wallet_data.encrypted_master_key, &wallet_data.salt, &wallet_data.nonce)?
        };

        // Replace encrypted key with decrypted key for use, will be zeroized on drop.
        wallet_data.encrypted_master_key = master_key;
        Ok(wallet_data)
    }

    fn encrypt_traditional(&self, data: &[u8], master_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        // Derive a dedicated encryption key from the master key to avoid reuse.
        let mut enc_key_bytes = [0u8; 32];
        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(b"enc-salt"), master_key);
        hkdf.expand(b"aes-gcm-key", &mut enc_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to derive encryption key: {}", e))?;

        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        use rand::RngCore;

        let key = Key::<Aes256Gcm>::from_slice(&enc_key_bytes);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("AES encryption failed: {}", e))?;
        Ok((ciphertext, b"enc-salt".to_vec(), nonce_bytes.to_vec()))
    }

    fn decrypt_traditional(&self, ciphertext: &[u8], salt: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };

        // Re-derive the same encryption key
        let mut enc_key_bytes = [0u8; 32];
        // This is a placeholder, in a real scenario the master_key for decryption would come from user input (password)
        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), ciphertext);
        hkdf.expand(b"aes-gcm-key", &mut enc_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to derive encryption key: {}", e))?;

        let key = Key::<Aes256Gcm>::from_slice(&enc_key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES decryption failed: {}", e))?;
        Ok(plaintext)
    }
}
