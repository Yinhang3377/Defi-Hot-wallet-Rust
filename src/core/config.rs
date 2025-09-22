use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub blockchain: BlockchainConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
    pub i18n: I18nConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub quantum_safe_default: bool,
    pub multi_sig_threshold: u8,
    pub hsm_enabled: bool,
    pub encryption_algorithm: String,
    pub key_derivation_rounds: u32,
    pub session_timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub networks: HashMap<String, NetworkConfig>,
    pub default_gas_limit: u64,
    pub transaction_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub explorer_url: String,
    pub native_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
    pub encryption_key_path: String,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub log_level: String,
    pub alert_webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub default_language: String,
    pub supported_languages: Vec<String>,
    pub resources_path: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        let mut blockchain_networks = HashMap::new();

        blockchain_networks.insert(
            "eth".to_string(),
            NetworkConfig {
                rpc_url: "https://rpc.ankr.com/eth".to_string(),
                chain_id: Some(1),
                explorer_url: "https://etherscan.io".to_string(),
                native_token: "ETH".to_string(),
            },
        );

        blockchain_networks.insert(
            "sepolia".to_string(),
            NetworkConfig {
                rpc_url: "https://rpc.sepolia.org".to_string(),
                chain_id: Some(11155111),
                explorer_url: "https://sepolia.etherscan.io".to_string(),
                native_token: "ETH".to_string(),
            },
        );

        blockchain_networks.insert(
            "solana".to_string(),
            NetworkConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                chain_id: None,
                explorer_url: "https://explorer.solana.com".to_string(),
                native_token: "SOL".to_string(),
            },
        );

        blockchain_networks.insert(
            "solana-devnet".to_string(),
            NetworkConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                chain_id: None,
                explorer_url: "https://explorer.solana.com/?cluster=devnet".to_string(),
                native_token: "SOL".to_string(),
            },
        );

        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                tls_enabled: false,
                cert_path: None,
                key_path: None,
            },
            security: SecurityConfig {
                quantum_safe_default: true,
                multi_sig_threshold: 2,
                hsm_enabled: false,
                encryption_algorithm: "Kyber1024".to_string(),
                key_derivation_rounds: 100000,
                session_timeout_minutes: 30,
            },
            blockchain: BlockchainConfig {
                networks: blockchain_networks,
                default_gas_limit: 21000,
                transaction_timeout_seconds: 300,
            },
            storage: StorageConfig {
                database_url: "sqlite://./data/wallet.db?mode=rwc".to_string(),
                encryption_key_path: "./keys/master.key".to_string(),
                backup_enabled: true,
                backup_interval_hours: 24,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                log_level: "info".to_string(),
                alert_webhook_url: None,
            },
            i18n: I18nConfig {
                default_language: "en".to_string(),
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                resources_path: "./resources/i18n".to_string(),
            },
        }
    }
}

impl WalletConfig {
    pub fn load_from_file(path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("WALLET"))
            .build()?;

        settings.try_deserialize()
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }
}
