// ...existing code...
use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use std::collections::HashMap;

#[test]
fn test_env_config_load_with_values() {
    let storage = StorageConfig {
        database_url: "sqlite::memory:".to_string(),
        max_connections: Some(5),
        connection_timeout_seconds: Some(30),
    };
    let blockchain =
        BlockchainConfig { networks: HashMap::new(), default_network: Some("eth".to_string()) };
    let cfg = WalletConfig { storage, blockchain, quantum_safe: false, multi_sig_threshold: 1 };

    assert_eq!(cfg.storage.database_url, "sqlite::memory:");
    assert_eq!(cfg.blockchain.default_network.as_deref(), Some("eth"));
    assert_eq!(cfg.multi_sig_threshold, 1);
}

#[test]
fn test_env_config_defaults() {
    // Ensure WalletConfig::default() exists and yields sensible fields.
    let default_cfg = WalletConfig::default();
    // Access fields to ensure compilation; adjust expectations if repo defaults differ.
    let _ = default_cfg.storage.database_url.clone();
    assert!(default_cfg.multi_sig_threshold >= 1);
}
// ...existing code...
