// tests/integration_tests.rs
use defi_hot_wallet::core::config::{
    BlockchainConfig, I18nConfig, MonitoringConfig, NetworkConfig, SecurityConfig, ServerConfig,
    StorageConfig, WalletConfig,
};
use defi_hot_wallet::core::wallet::WalletManager;
use defi_hot_wallet::storage::{WalletStorage, WalletStorageTrait};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_full_wallet_workflow() {
    // Initialize storage
    let _storage: Arc<dyn WalletStorageTrait> =
        Arc::new(WalletStorage::new_with_url("sqlite::memory:").await.unwrap());

    // Create wallet manager
    let manager = WalletManager::new(&create_mock_config()).await.unwrap();

    // Generate mnemonic
    let mnemonic = manager.generate_mnemonic().unwrap();
    assert!(!mnemonic.is_empty());

    // Derive master key
    let master_key = manager.derive_master_key(&mnemonic).unwrap();
    assert_eq!(master_key.len(), 32);

    // Derive addresses for different chains
    let eth_address = manager.derive_address(&master_key, "eth").unwrap();
    assert!(eth_address.starts_with("0x"));

    let sol_address = manager.derive_address(&master_key, "solana").unwrap();
    assert!(!sol_address.is_empty());

    // Test bridge fee calculation
    let (fee, estimated_time) = manager
        .calculate_bridge_fee("eth", "solana", "USDC", "100.0")
        .unwrap();
    assert_eq!(fee, "1"); // 1% of 100.0 is 1.0, which to_string() is "1" for f64
    assert!(estimated_time > chrono::Utc::now());
}

#[tokio::test]
async fn test_cross_chain_operations() {
    let manager = WalletManager::new(&create_mock_config()).await.unwrap();

    // Test multiple chain pairs
    let chain_pairs = vec![("eth", "solana"), ("solana", "eth"), ("eth", "bsc")];

    for (from, to) in chain_pairs {
        let (fee, _) = manager
            .calculate_bridge_fee(from, to, "USDC", "50.0")
            .unwrap();
        assert_eq!(fee, "0.5"); // 1% of 50.0
    }
}

#[tokio::test]
async fn test_error_recovery() {
    let manager = WalletManager::new(&create_mock_config()).await.unwrap();

    // Test invalid network
    let result = manager.derive_address(&[1u8; 32], "invalid");
    assert!(result.is_err());

    // Test invalid mnemonic
    let result = manager.derive_master_key("");
    assert!(result.is_err());

    // Test invalid amount (this should fail parsing)
    let result = manager.calculate_bridge_fee("eth", "solana", "USDC", "invalid-amount");
    assert!(result.is_err());
}

fn create_mock_config() -> WalletConfig {
    let mut blockchain_networks = HashMap::new();
    blockchain_networks.insert(
        "eth".to_string(),
        NetworkConfig {
            rpc_url: "https://mock-eth.com".to_string(),
            chain_id: Some(1),
            explorer_url: "".to_string(),
            native_token: "ETH".to_string(),
        },
    );
    blockchain_networks.insert(
        "solana".to_string(),
        NetworkConfig {
            rpc_url: "https://mock-solana.com".to_string(),
            chain_id: None,
            explorer_url: "".to_string(),
            native_token: "SOL".to_string(),
        },
    );

    WalletConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            tls_enabled: false,
            cert_path: None,
            key_path: None,
        },
        security: SecurityConfig {
            quantum_safe_default: false,
            multi_sig_threshold: 2,
            hsm_enabled: false,
            encryption_algorithm: "AES-GCM".to_string(),
            key_derivation_rounds: 10000,
            session_timeout_minutes: 30,
        },
        blockchain: BlockchainConfig {
            networks: blockchain_networks,
            default_gas_limit: 21000,
            transaction_timeout_seconds: 300,
        },
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            encryption_key_path: "/tmp/test_key".to_string(),
            backup_enabled: false,
            backup_interval_hours: 24,
        },
        monitoring: MonitoringConfig {
            metrics_enabled: false,
            metrics_port: 9090,
            log_level: "info".to_string(),
            alert_webhook_url: None,
        },
        i18n: I18nConfig {
            default_language: "en".to_string(),
            supported_languages: vec!["en".to_string()],
            resources_path: "resources".to_string(),
        },
    }
}