use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a blockchain network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub native_token: String,
    pub block_time_seconds: u64,
}

/// Configuration for storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub connection_timeout_seconds: Option<u64>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:wallets.db".to_string(),
            max_connections: Some(10),
            connection_timeout_seconds: Some(30),
        }
    }
}

/// Configuration for blockchain networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub networks: HashMap<String, NetworkConfig>,
    pub default_network: Option<String>,
}

/// Main wallet configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub storage: StorageConfig,
    pub blockchain: BlockchainConfig,
    pub quantum_safe: bool,
    pub multi_sig_threshold: u8,
}

impl Default for WalletConfig {
    fn default() -> Self {
        let mut networks = HashMap::new();
        networks.insert(
            "eth".to_string(),
            NetworkConfig {
                rpc_url: "https://ethereum.publicnode.com".to_string(),
                chain_id: Some(1),
                native_token: "ETH".to_string(),
                block_time_seconds: 12,
            },
        );
        networks.insert(
            "sepolia".to_string(),
            NetworkConfig {
                rpc_url: "https://ethereum-sepolia.publicnode.com".to_string(),
                chain_id: Some(11155111),
                native_token: "ETH".to_string(),
                block_time_seconds: 12,
            },
        );
        networks.insert(
            "solana".to_string(),
            NetworkConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                chain_id: None,
                native_token: "SOL".to_string(),
                block_time_seconds: 1,
            },
        );
        networks.insert(
            "solana-devnet".to_string(),
            NetworkConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                chain_id: None,
                native_token: "SOL".to_string(),
                block_time_seconds: 1,
            },
        );
        networks.insert(
            "polygon".to_string(),
            NetworkConfig {
                rpc_url: "https://polygon-rpc.com".to_string(),
                chain_id: Some(137),
                native_token: "MATIC".to_string(),
                block_time_seconds: 2,
            },
        );
        networks.insert(
            "bsc".to_string(),
            NetworkConfig {
                rpc_url: "https://bsc-dataseed.bnbchain.org/".to_string(),
                chain_id: Some(56),
                native_token: "BNB".to_string(),
                block_time_seconds: 3,
            },
        );

        Self {
            storage: StorageConfig {
                database_url: "sqlite:wallets.db".to_string(),
                max_connections: Some(10),
                connection_timeout_seconds: Some(30),
            },
            blockchain: BlockchainConfig { networks, default_network: Some("eth".to_string()) },
            quantum_safe: false,
            multi_sig_threshold: 2,
        }
    }
}

impl WalletConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.storage.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }
        if self.blockchain.networks.is_empty() {
            return Err(anyhow::anyhow!("At least one network must be configured"));
        }
        for (name, config) in &self.blockchain.networks {
            if config.rpc_url.is_empty() {
                return Err(anyhow::anyhow!("RPC URL for network '{}' cannot be empty", name));
            }
        }
        Ok(())
    }

    /// Loads configuration from a TOML file.
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Saves configuration to a TOML file.
    pub fn to_file(&self, path: &str) -> Result<()> {
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WalletConfig::default();
        assert_eq!(config.storage.database_url, "sqlite:wallets.db");
        assert!(config.blockchain.networks.contains_key("eth"));
        assert_eq!(config.multi_sig_threshold, 2);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = WalletConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_database_url() {
        let mut config = WalletConfig::default();
        config.storage.database_url = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_no_networks() {
        let mut config = WalletConfig::default();
        config.blockchain.networks.clear();
        assert!(config.validate().is_err());
    }
}
