use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::blockchain::{ethereum::EthereumClient, solana::SolanaClient, traits::BlockchainClient};
use crate::crypto::{
    hsm::HSMManager, multisig::MultiSignature, quantum::QuantumSafeEncryption,
    shamir::ShamirSecretSharing,
};
use crate::core::config::WalletConfig;
use crate::storage::WalletStorage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub quantum_safe: bool,
    pub multi_sig_threshold: u8,
    pub networks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecureWalletData {
    #[zeroize(skip)]
    pub info: WalletInfo,
    pub mnemonic: String,
    pub master_key: Vec<u8>,
    pub shamir_shares: Vec<Vec<u8>>,
}

pub struct WalletManager {
    storage: WalletStorage,
    quantum_crypto: QuantumSafeEncryption,
    shamir: ShamirSecretSharing,
    _multisig: MultiSignature,
    _hsm: HSMManager,
    blockchain_clients: Arc<HashMap<String, Box<dyn BlockchainClient>>>,
}

impl WalletManager {
    pub async fn new(config: &WalletConfig) -> Result<Self> {
        info!("🔧 Initializing WalletManager");

        let storage = WalletStorage::new().await?;
        let quantum_crypto = QuantumSafeEncryption::new()?;
        let shamir = ShamirSecretSharing::new();
        let multisig = MultiSignature::new();
        let hsm = HSMManager::new().await?;

        let mut blockchain_clients: HashMap<String, Box<dyn BlockchainClient>> = HashMap::new();

        for (name, network_config) in &config.blockchain.networks {
            info!("Initializing client for network: {}", name);
            let client: Result<Box<dyn BlockchainClient>> = match name.as_str() {
                "eth" | "sepolia" | "polygon" | "bsc" | "bsctestnet" => EthereumClient::new(&network_config.rpc_url)
                    .await
                    .map(|c| Box::new(c) as Box<dyn BlockchainClient>),
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
                    info!("✅ {} client initialized for network '{}'", native_token, name);
                },
                Err(e) => warn!("⚠️ Failed to initialize client for {}: {}", name, e),
            }
        }

        Ok(Self {
            storage,
            quantum_crypto,
            shamir,
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
        let master_key = self.derive_master_key(&mnemonic)?;

        // Create Shamir secret shares (2-of-3 threshold)
        let shamir_shares = self.shamir.create_shares(&master_key, 3, 2)?;

        // Create wallet info
        let wallet_info = WalletInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            quantum_safe,
            multi_sig_threshold: 2,
            networks: vec!["eth".to_string(), "solana".to_string()],
        };

        // Create secure wallet data
        let mut wallet_data = SecureWalletData {
            info: wallet_info.clone(),
            mnemonic,
            master_key,
            shamir_shares,
        };

        // Encrypt and store wallet
        self.store_wallet_securely(&wallet_data, quantum_safe)
            .await?;

        // Clear sensitive data from memory
        wallet_data.zeroize();

        info!("✅ Wallet '{}' created with ID: {}", name, wallet_info.id);
        Ok(wallet_info)
    }

    pub async fn get_balance(&self, wallet_name: &str, network: &str) -> Result<String> {
        info!(
            "💰 Getting balance for wallet: {} on network: {}",
            wallet_name, network
        );

        // Load wallet
        let wallet_data = self.load_wallet_securely(wallet_name).await?;

        // Get blockchain client
        let client = self
            .blockchain_clients
            .get(network)
            .ok_or_else(|| anyhow::anyhow!("Unsupported network: {}", network))?;

        // Derive address for the network
        let address = self.derive_address(&wallet_data.master_key, network)?;

        // Get balance from blockchain
        let balance = client.get_balance(&address).await?;

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
        let wallet_data = self.load_wallet_securely(wallet_name).await?;

        // Get blockchain client
        let client = self
            .blockchain_clients
            .get(network)
            .ok_or_else(|| anyhow::anyhow!("Unsupported network: {}", network))?;

        // Create and sign transaction
        let private_key = self.derive_private_key(&wallet_data.master_key, network)?;
        let tx_hash = client
            .send_transaction(&private_key, to_address, amount)
            .await?;

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

    fn derive_master_key(&self, mnemonic: &str) -> Result<Vec<u8>> {
        use bip32::Seed;
        use bip39::{Language, Mnemonic};

        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)?;
        let seed_bytes = mnemonic.to_seed("");
        let seed = Seed::new(seed_bytes);
        Ok(seed.as_bytes().to_vec())
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
        wallet_data: &SecureWalletData,
        quantum_safe: bool,
    ) -> Result<()> {
        let encrypted_data = if quantum_safe {
            self.quantum_crypto
                .encrypt(&bincode::serialize(wallet_data)?)?
        } else {
            // Use traditional AES-GCM encryption as fallback
            self.encrypt_traditional(&bincode::serialize(wallet_data)?)?
        };

        self.storage
            .store_wallet(&wallet_data.info.name, &encrypted_data)
            .await?;
        Ok(())
    }

    async fn load_wallet_securely(&self, wallet_name: &str) -> Result<SecureWalletData> {
        let encrypted_data = self.storage.load_wallet(wallet_name).await?;

        // Try quantum-safe decryption first, fallback to traditional
        let decrypted_data = match self.quantum_crypto.decrypt(&encrypted_data) {
            Ok(data) => data,
            Err(_) => self.decrypt_traditional(&encrypted_data)?,
        };

        let wallet_data: SecureWalletData = bincode::deserialize(&decrypted_data)?;
        Ok(wallet_data)
    }

    fn encrypt_traditional(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simplified traditional encryption - in production use proper key derivation
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        use rand::RngCore;

        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]); // In production, derive from user password/HSM
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut result = nonce_bytes.to_vec();
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("AES encryption failed: {}", e))?;
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_traditional(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };

        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }

        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        let cipher = Aes256Gcm::new(key);

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES decryption failed: {}", e))?;
        Ok(plaintext)
    }
}
