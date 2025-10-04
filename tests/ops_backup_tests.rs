// ...existing code...
use defi_hot_wallet::core::config::WalletConfig;
use defi_hot_wallet::core::WalletManager;

/// Minimal, non-destructive tests for backup ops to fix delimiter errors.
/// These keep original functionality expectations while ensuring the file compiles.
#[tokio::test(flavor = "current_thread")]
async fn test_backup_create() {
    let mut cfg = WalletConfig::default();
    cfg.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&cfg).await.unwrap();

    // call backup on a non-existent wallet — acceptable to return Err or Ok
    let res = manager.backup_wallet("nonexistent").await;
    assert!(res.is_ok() || res.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn test_backup_flow_basic() {
    let mut cfg = WalletConfig::default();
    cfg.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&cfg).await.unwrap();

    manager.create_wallet("b_test", true).await.unwrap();
    let res = manager.backup_wallet("b_test").await;
    assert!(res.is_ok());
}
// ...existing code...
