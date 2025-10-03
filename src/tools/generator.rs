// src/core/config.rs
//! 閰嶇疆绠＄悊妯″潡
//! 鎻愪緵閰嶇疆鏂囦欢鐨勫姞杞姐€佷繚瀛樸€侀獙璇佸拰绠＄悊鍔熻兘

use crate::tools::error::WalletError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 搴旂敤绋嬪簭閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 搴旂敤绋嬪簭鍩烘湰淇℃伅
    pub app: AppConfig,
    /// 鍖哄潡閾剧綉缁滈厤缃?    pub blockchain: BlockchainConfig,
    /// 瀹夊叏閰嶇疆
    pub security: SecurityConfig,
    /// 瀛樺偍閰嶇疆
    pub storage: StorageConfig,
    /// 鐩戞帶閰嶇疆
    pub monitoring: MonitoringConfig,
    /// 鍥介檯鍖栭厤缃?    pub i18n: I18nConfig,
}

/// 搴旂敤绋嬪簭鍩烘湰閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 搴旂敤绋嬪簭鍚嶇О
    pub name: String,
    /// 鐗堟湰
    pub version: String,
    /// 鐜
    pub environment: String,
    /// 璋冭瘯妯″紡
    pub debug: bool,
    /// 鏃ュ織绾у埆
    pub log_level: String,
}

/// 鍖哄潡閾剧綉缁滈厤缃?#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// 榛樿缃戠粶
    pub default_network: String,
    /// 缃戠粶閰嶇疆鍒楄〃
    pub networks: HashMap<String, NetworkConfig>,
}

/// 缃戠粶閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 缃戠粶鍚嶇О
    pub name: String,
    /// RPC URL
    pub rpc_url: String,
    /// 閾綢D
    pub chain_id: u64,
    /// 璐у竵绗﹀彿
    pub symbol: String,
    /// 鍖哄潡娴忚鍣║RL
    pub explorer_url: Option<String>,
    /// 纭鍧楁暟
    pub confirmations: u64,
}

/// 瀹夊叏閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 鍔犲瘑绠楁硶
    pub encryption_algorithm: String,
    /// 瀵嗛挜娲剧敓鍑芥暟
    pub kdf_algorithm: String,
    /// 瀵嗙爜鏈€灏忛暱搴?    pub min_password_length: usize,
    /// 浼氳瘽瓒呮椂鏃堕棿锛堢锛?    pub session_timeout: u64,
    /// 鏈€澶х櫥褰曞皾璇曟鏁?    pub max_login_attempts: u32,
    /// 閿佸畾鏃堕棿锛堢锛?    pub lockout_duration: u64,
    /// 鍚敤鍙屽洜绱犺璇?    pub enable_2fa: bool,
    /// 鍚堣妫€鏌?    pub compliance: ComplianceConfig,
}

/// 鍚堣閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// 鍚敤鍚堣妫€鏌?    pub enabled: bool,
    /// 鍙楅檺鍥藉鍒楄〃
    pub restricted_countries: Vec<String>,
    /// 鍙楅檺鍦板潃鍒楄〃
    pub sanctioned_addresses: Vec<String>,
    /// 浜ゆ槗闄愰
    pub transaction_limits: HashMap<String, f64>,
    /// KYC瑕佹眰
    pub require_kyc: bool,
}

/// 瀛樺偍閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 鏁版嵁搴撶被鍨?    pub database_type: String,
    /// 鏁版嵁搴揢RL
    pub database_url: String,
    /// 杩炴帴姹犲ぇ灏?    pub connection_pool_size: u32,
    /// 缂撳瓨澶у皬
    pub cache_size: usize,
    /// 澶囦唤闂撮殧锛堢锛?    pub backup_interval: u64,
    /// 淇濈暀澶囦唤鏁伴噺
    pub backup_retention: u32,
}

/// 鐩戞帶閰嶇疆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 鍚敤鐩戞帶
    pub enabled: bool,
    /// 鎸囨爣鏀堕泦闂撮殧锛堢锛?    pub metrics_interval: u64,
    /// 鍋ュ悍妫€鏌ラ棿闅旓紙绉掞級
    pub health_check_interval: u64,
    /// 璀︽姤闃堝€?    pub alert_thresholds: HashMap<String, f64>,
    /// 鏃ュ織杞浆澶у皬锛圡B锛?    pub log_rotation_size: u64,
    /// 淇濈暀鏃ュ織澶╂暟
    pub log_retention_days: u32,
}

/// 鍥介檯鍖栭厤缃?#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    /// 榛樿璇█
    pub default_language: String,
    /// 鏀寔鐨勮瑷€鍒楄〃
    pub supported_languages: Vec<String>,
    /// 缈昏瘧鏂囦欢璺緞
    pub translation_path: String,
    /// 鏃跺尯
    pub timezone: String,
}

/// 閰嶇疆绠＄悊鍣?pub struct ConfigManager {
    config: Config,
    config_path: String,
}

impl Default for ConfigManager {
    /// Creates a new `ConfigManager` with a default configuration file name "config.json".
    fn default() -> Self {
        Self::new("config.json")
    }
}

impl ConfigManager {
    /// 鍒涘缓鏂扮殑閰嶇疆绠＄悊鍣?    pub fn new(config_path: impl Into<String>) -> Self {
        Self { config: Config::default(), config_path: config_path.into() }
    }

    /// 鍔犺浇閰嶇疆
    pub fn load(&mut self) -> Result<(), WalletError> {
        if !Path::new(&self.config_path).exists() {
            // 濡傛灉閰嶇疆鏂囦欢涓嶅瓨鍦紝鍒涘缓榛樿閰嶇疆
            self.config = Config::default();
            self.save()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path).map_err(WalletError::IoError)?;

        self.config = serde_json::from_str(&content)
            .map_err(|e| WalletError::DeserializationError(e.to_string()))?;

        Ok(())
    }

    /// 淇濆瓨閰嶇疆
    pub fn save(&self) -> Result<(), WalletError> {
        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        // 纭繚鐩綍瀛樺湪
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent).map_err(WalletError::IoError)?;
        }

        fs::write(&self.config_path, content).map_err(WalletError::IoError)?;

        Ok(())
    }

    /// 鑾峰彇閰嶇疆
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// 鑾峰彇鍙彉閰嶇疆
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// 璁剧疆閰嶇疆
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// 楠岃瘉閰嶇疆
    pub fn validate(&self) -> Result<(), WalletError> {
        self.config.validate()
    }

    /// 閲嶇疆涓洪粯璁ら厤缃?    pub fn reset_to_default(&mut self) {
        self.config = Config::default();
    }

    /// 鑾峰彇閰嶇疆璺緞
    pub fn config_path(&self) -> &str {
        &self.config_path
    }
}

impl Default for Config {
    /// Creates a default configuration.
    fn default() -> Self {
        let mut networks = HashMap::new();

        // Ethereum Mainnet
        networks.insert(
            "ethereum_mainnet".to_string(),
            NetworkConfig {
                name: "Ethereum Mainnet".to_string(),
                rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
                chain_id: 1,
                symbol: "ETH".to_string(),
                explorer_url: Some("https://etherscan.io".to_string()),
                confirmations: 12,
            },
        );

        // Solana Mainnet
        networks.insert(
            "solana_mainnet".to_string(),
            NetworkConfig {
                name: "Solana Mainnet".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                chain_id: 101,
                symbol: "SOL".to_string(),
                explorer_url: Some("https://solscan.io".to_string()),
                confirmations: 32,
            },
        );

        Self {
            app: AppConfig {
                name: "DeFi Hot Wallet".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: "development".to_string(),
                debug: false,
                log_level: "info".to_string(),
            },
            blockchain: BlockchainConfig {
                default_network: "ethereum_mainnet".to_string(),
                networks,
            },
            security: SecurityConfig {
                encryption_algorithm: "AES-256-GCM".to_string(),
                kdf_algorithm: "PBKDF2".to_string(),
                min_password_length: 8,
                session_timeout: 3600, // 1 hour
                max_login_attempts: 5,
                lockout_duration: 900, // 15 minutes
                enable_2fa: true,
                compliance: ComplianceConfig {
                    enabled: true,
                    restricted_countries: vec!["US".to_string(), "CN".to_string()],
                    sanctioned_addresses: vec![],
                    transaction_limits: {
                        let mut limits = HashMap::new();
                        limits.insert("daily".to_string(), 10000.0);
                        limits.insert("monthly".to_string(), 50000.0);
                        limits
                    },
                    require_kyc: false,
                },
            },
            storage: StorageConfig {
                database_type: "SQLite".to_string(),
                database_url: "wallet.db".to_string(),
                connection_pool_size: 10,
                cache_size: 1000,
                backup_interval: 86400, // 1 day
                backup_retention: 30,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_interval: 60,
                health_check_interval: 300,
                alert_thresholds: {
                    let mut thresholds = HashMap::new();
                    thresholds.insert("cpu_usage".to_string(), 80.0);
                    thresholds.insert("memory_usage".to_string(), 90.0);
                    thresholds
                },
                log_rotation_size: 100, // 100 MB
                log_retention_days: 30,
            },
            i18n: I18nConfig {
                default_language: "en".to_string(),
                supported_languages: vec!["en".to_string(), "zh".to_string(), "es".to_string()],
                translation_path: "translations".to_string(),
                timezone: "UTC".to_string(),
            },
        }
    }
}

impl Config {
    /// 楠岃瘉閰嶇疆
    pub fn validate(&self) -> Result<(), WalletError> {
        // 楠岃瘉搴旂敤绋嬪簭閰嶇疆
        if self.app.name.is_empty() {
            return Err(WalletError::InvalidInput("App name cannot be empty".to_string()));
        }

        if self.app.version.is_empty() {
            return Err(WalletError::InvalidInput("App version cannot be empty".to_string()));
        }

        // 楠岃瘉鍖哄潡閾鹃厤缃?        if self.blockchain.networks.is_empty() {
            return Err(WalletError::InvalidInput(
                "At least one network must be configured".to_string(),
            ));
        }

        if !self.blockchain.networks.contains_key(&self.blockchain.default_network) {
            return Err(WalletError::InvalidInput(
                "Default network not found in networks".to_string(),
            ));
        }

        // 楠岃瘉瀹夊叏閰嶇疆
        if self.security.min_password_length < 8 {
            return Err(WalletError::InvalidInput(
                "Minimum password length must be at least 8".to_string(),
            ));
        }

        // 楠岃瘉瀛樺偍閰嶇疆
        if self.storage.database_url.is_empty() {
            return Err(WalletError::InvalidInput("Database URL cannot be empty".to_string()));
        }

        // 楠岃瘉鐩戞帶閰嶇疆
        if self.monitoring.enabled && self.monitoring.metrics_interval == 0 {
            return Err(WalletError::InvalidInput("Metrics interval cannot be zero".to_string()));
        }

        // 楠岃瘉鍥介檯鍖栭厤缃?        if self.i18n.supported_languages.is_empty() {
            return Err(WalletError::InvalidInput(
                "At least one supported language must be specified".to_string(),
            ));
        }

        if !self.i18n.supported_languages.contains(&self.i18n.default_language) {
            return Err(WalletError::InvalidInput(
                "Default language must be in supported languages".to_string(),
            ));
        }

        Ok(())
    }

    /// 鑾峰彇缃戠粶閰嶇疆
    pub fn get_network(&self, network_name: &str) -> Option<&NetworkConfig> {
        self.blockchain.networks.get(network_name)
    }

    /// 鑾峰彇榛樿缃戠粶閰嶇疆
    pub fn get_default_network(&self) -> &NetworkConfig {
        self.blockchain
            .networks
            .get(&self.blockchain.default_network)
            .expect("Default network should exist")
    }

    /// 妫€鏌ュ湴鍧€鏄惁鍙楅檺
    pub fn is_address_restricted(&self, address: &str) -> bool {
        self.security.compliance.enabled
            && self
                .security
                .compliance
                .sanctioned_addresses
                .iter()
                .any(|restricted| restricted.eq_ignore_ascii_case(address))
    }

    /// 妫€鏌ュ浗瀹舵槸鍚﹀彈闄?    pub fn is_country_restricted(&self, country: &str) -> bool {
        self.security.compliance.enabled
            && self
                .security
                .compliance
                .restricted_countries
                .iter()
                .any(|restricted| restricted.eq_ignore_ascii_case(country))
    }

    /// 鑾峰彇浜ゆ槗闄愰
    pub fn get_transaction_limit(&self, period: &str) -> Option<f64> {
        if !self.security.compliance.enabled {
            return None;
        }

        self.security.compliance.transaction_limits.get(period).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        // Test invalid config
        let mut invalid_config = config.clone();
        invalid_config.app.name = "".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_loading_and_saving() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        // Create and save config
        let manager = ConfigManager::new(config_path.to_str().unwrap());
        manager.save().unwrap();

        // Load config
        let mut new_manager = ConfigManager::new(config_path.to_str().unwrap());
        new_manager.load().unwrap();

        // Verify configs are equal
        assert_eq!(manager.get_config().app.name, new_manager.get_config().app.name);
        assert_eq!(manager.get_config().app.version, new_manager.get_config().app.version);
    }

    #[test]
    fn test_network_config() {
        let config = Config::default();

        let eth_network = config.get_network("ethereum_mainnet").unwrap();
        assert_eq!(eth_network.chain_id, 1);
        assert_eq!(eth_network.symbol, "ETH");

        let default_network = config.get_default_network();
        assert_eq!(default_network.name, "Ethereum Mainnet");
    }

    #[test]
    fn test_compliance_checks() {
        let config = Config::default();

        // Test restricted address
        assert!(!config.is_address_restricted("0x1234567890abcdef"));

        // Test restricted countries - US and CN are restricted by default
        assert!(config.is_country_restricted("US"));
        assert!(config.is_country_restricted("CN"));
        assert!(!config.is_country_restricted("JP")); // Japan is not restricted

        // Test transaction limits
        assert_eq!(config.get_transaction_limit("daily"), Some(10000.0));
        assert_eq!(config.get_transaction_limit("monthly"), Some(50000.0));
        assert_eq!(config.get_transaction_limit("nonexistent"), None);
    }

    #[test]
    fn test_config_modification() {
        let mut config = Config::default();

        // Modify config
        config.app.debug = true;
        config.security.min_password_length = 12;

        // Validate modified config
        assert!(config.validate().is_ok());
        assert!(config.app.debug);
        assert_eq!(config.security.min_password_length, 12);
    }
}
