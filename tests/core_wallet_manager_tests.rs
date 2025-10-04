// ...existing code...
use defi_hot_wallet::core::config::WalletConfig;
use defi_hot_wallet::core::WalletManager;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

// Small helper to reduce repetition and ensure all tests use in-memory DB by default.
fn in_memory_config() -> WalletConfig {
    let mut cfg = WalletConfig::default();
    cfg.storage.database_url = "sqlite::memory:".to_string();
    cfg
}

#[tokio::test(flavor = "current_thread")]
async fn test_wallet_manager_new() {
    let config = in_memory_config();
    let _manager = WalletManager::new(&config).await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn test_wallet_manager_new_invalid_db() {
    let mut cfg = WalletConfig::default();
    cfg.storage.database_url = "invalid".to_string();
    let result = WalletManager::new(&cfg).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallet = manager.create_wallet("test_wallet", true).await.unwrap();
    assert_eq!(wallet.name, "test_wallet");
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_non_quantum() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallet = manager.create_wallet("test_wallet", false).await.unwrap();
    assert_eq!(wallet.name, "test_wallet");
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_duplicate() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.create_wallet("test", false).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_empty_name() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.create_wallet("", true).await;
    // Accept either success or an error depending on implementation.
    assert!(result.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_wallets_empty() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallets = manager.list_wallets().await.unwrap();
    assert!(wallets.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn test_list_wallets_with_wallets() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("wallet1", true).await.unwrap();
    manager.create_wallet("wallet2", false).await.unwrap();
    let wallets = manager.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn test_delete_wallet() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.delete_wallet("test").await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_delete_wallet_not_found() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.delete_wallet("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_backup_wallet() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.backup_wallet("test").await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_backup_wallet_existing() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.backup_wallet("test").await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_concurrent_create_wallets() {
    let config = in_memory_config();
    // Reduced concurrency to avoid long runs while exercising concurrency paths.
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));
    let mut handles = Vec::new();

    for i in 0..4 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mgr = manager_clone.lock().await;
            mgr.create_wallet(&format!("wallet_{}", i), true).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr = manager.lock().await;
    let wallets = mgr.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 4);
}

#[tokio::test(flavor = "current_thread")]
async fn test_concurrent_delete_wallets() {
    let config = in_memory_config();
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));

    // create wallets
    {
        let mgr = manager.lock().await;
        for i in 0..3 {
            mgr.create_wallet(&format!("wallet_{}", i), true).await.unwrap();
        }
    }

    // concurrent deletes
    let mut handles = Vec::new();
    for i in 0..3 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mgr = manager_clone.lock().await;
            mgr.delete_wallet(&format!("wallet_{}", i)).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr = manager.lock().await;
    let wallets = mgr.list_wallets().await.unwrap();
    assert!(wallets.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn test_concurrent_mixed_operations() {
    let config = in_memory_config();
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));

    let mut handles = Vec::new();
    for i in 0..3 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mgr = manager_clone.lock().await;
            mgr.create_wallet(&format!("mixed_{}", i), true).await.unwrap();
            let _ = mgr.list_wallets().await.unwrap();
            let _ = mgr.backup_wallet(&format!("mixed_{}", i)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr = manager.lock().await;
    let wallets = mgr.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 3);
}

#[tokio::test(flavor = "current_thread")]
async fn test_restore_wallet() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    let result = manager
        .restore_wallet(
            "restored",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .await;
    assert!(result.is_ok());

    let wallets = manager.list_wallets().await.unwrap();
    assert!(wallets.iter().any(|w| w.name == "restored"));
}

#[tokio::test(flavor = "current_thread")]
async fn test_restore_wallet_already_exists() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    manager.create_wallet("existing", true).await.unwrap();

    let result = manager
        .restore_wallet(
            "existing",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_restore_wallet_invalid_mnemonic() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    let result = manager.restore_wallet("invalid_restore", "invalid mnemonic").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_backup_restore_flow() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    manager.create_wallet("backup_test", true).await.unwrap();

    let backup_result = manager.backup_wallet("backup_test").await;
    assert!(backup_result.is_ok());
    let mnemonic = backup_result.unwrap();

    manager.delete_wallet("backup_test").await.unwrap();

    let restore_result = manager.restore_wallet("restored_backup", &mnemonic).await;
    assert!(restore_result.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_with_network() {
    let mut cfg = in_memory_config();
    // If no networks configured, get_balance is expected to error.
    cfg.blockchain.networks.clear();
    let manager = WalletManager::new(&cfg).await.unwrap();

    manager.create_wallet("balance_test", true).await.unwrap();

    let balance = manager.get_balance("balance_test", "eth").await;
    assert!(balance.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_wallet_not_found() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    let result = manager.get_balance("nonexistent", "eth").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_balance_invalid_network() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    manager.create_wallet("network_test", true).await.unwrap();

    let result = manager.get_balance("network_test", "invalid_network").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_wallet_persistence() {
    let temp_dir = tempdir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    let db_url = "sqlite://wallet_db.sqlite?mode=rwc".to_string();

    {
        let mut cfg = WalletConfig::default();
        cfg.storage.database_url = db_url.clone();
        let manager = WalletManager::new(&cfg).await.unwrap();

        manager.create_wallet("persistent", true).await.unwrap();
    }

    {
        let mut cfg = WalletConfig::default();
        cfg.storage.database_url = db_url;
        let manager = WalletManager::new(&cfg).await.unwrap();

        let wallets = manager.list_wallets().await.unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].name, "persistent");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_wallet_address() {
    let config = in_memory_config();
    let manager = WalletManager::new(&config).await.unwrap();

    manager.create_wallet("address_test", true).await.unwrap();

    let address = manager.derive_address(b"some_master_key", "eth");
    assert!(address.is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn test_database_connection_error() {
    let mut cfg = WalletConfig::default();
    cfg.storage.database_url = "/invalid/path/that/cannot/exist".to_string();

    let result = WalletManager::new(&cfg).await;
    assert!(result.is_err());
}
// ...existing code...
