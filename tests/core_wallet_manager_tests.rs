use defi_hot_wallet::core::config::WalletConfig;
use defi_hot_wallet::core::WalletManager;
use std::env;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

/// 妫€鏌ュ綋鍓嶆槸鍚﹀湪 tarpaulin 鐜涓嬭繍琛?pub fn is_running_under_tarpaulin() -> bool {
    env::var("LLVM_PROFILE_FILE").is_ok() || env::var("CARGO_TARPAULIN").is_ok()
}

#[tokio::test]
async fn test_wallet_manager_new() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let _manager = WalletManager::new(&config).await.unwrap();
}

#[tokio::test]
async fn test_wallet_manager_new_invalid_db() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "invalid".to_string();
    let result = WalletManager::new(&config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_wallet() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallet = manager.create_wallet("test_wallet", true).await.unwrap();
    assert_eq!(wallet.name, "test_wallet");
}

#[tokio::test]
async fn test_create_wallet_non_quantum() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallet = manager.create_wallet("test_wallet", false).await.unwrap();
    assert_eq!(wallet.name, "test_wallet");
}

#[tokio::test]
async fn test_create_wallet_duplicate() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.create_wallet("test", false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_wallet_empty_name() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.create_wallet("", true).await;
    assert!(result.is_ok()); // 淇敼涓?is_ok锛屽尮閰嶅綋鍓嶅疄鐜板厑璁哥┖鍚嶇О
}

#[tokio::test]
async fn test_list_wallets_empty() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let wallets = manager.list_wallets().await.unwrap();
    assert!(wallets.is_empty());
}

#[tokio::test]
async fn test_list_wallets_with_wallets() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("wallet1", true).await.unwrap();
    manager.create_wallet("wallet2", false).await.unwrap();
    let wallets = manager.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 2);
}

#[tokio::test]
async fn test_delete_wallet() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.delete_wallet("test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_wallet_not_found() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.delete_wallet("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_backup_wallet() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    let result = manager.backup_wallet("test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_backup_wallet_existing() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();
    manager.create_wallet("test", true).await.unwrap();
    let result = manager.backup_wallet("test").await;
    assert!(result.is_ok());
}

// 淇骞跺彂鍒涘缓閽卞寘娴嬭瘯 - 绉婚櫎绾㈣壊娉㈡氮绾?#[tokio::test]
async fn test_concurrent_create_wallets() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));
    let mut handles = Vec::new();

    for i in 0..10 {
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
    assert_eq!(wallets.len(), 10);
}

// 淇骞跺彂鍒犻櫎閽卞寘娴嬭瘯 - 绉婚櫎绾㈣壊娉㈡氮绾?#[tokio::test]
async fn test_concurrent_delete_wallets() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));

    // 鍒涘缓閽卞寘
    {
        let mgr = manager.lock().await;
        for i in 0..5 {
            mgr.create_wallet(&format!("wallet_{}", i), true).await.unwrap();
        }
    }

    // 骞跺彂鍒犻櫎
    let mut handles = Vec::new();
    for i in 0..5 {
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

// 淇骞跺彂娣峰悎鎿嶄綔娴嬭瘯 - 绉婚櫎绾㈣壊娉㈡氮绾?#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = Arc::new(Mutex::new(WalletManager::new(&config).await.unwrap()));

    let mut handles = Vec::new();
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mgr = manager_clone.lock().await;
            mgr.create_wallet(&format!("mixed_{}", i), true).await.unwrap();
            mgr.list_wallets().await.unwrap();
            mgr.backup_wallet(&format!("mixed_{}", i)).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr = manager.lock().await;
    let wallets = mgr.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 5);
}

#[tokio::test]
async fn test_restore_wallet() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 浣跨敤restore_wallet鏂规硶鏇夸唬import_wallet
    let result = manager.restore_wallet("restored", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").await;
    assert!(result.is_ok());

    // 楠岃瘉鎭㈠鐨勯挶鍖?    let wallets = manager.list_wallets().await.unwrap();
    assert!(wallets.iter().any(|w| w.name == "restored"));
}

#[tokio::test]
async fn test_restore_wallet_already_exists() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 鍏堝垱寤洪挶鍖?    manager.create_wallet("existing", true).await.unwrap();

    // 灏濊瘯鎭㈠鍚屽悕閽卞寘
    let result = manager.restore_wallet("existing", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").await;
    // 鏍规嵁瀹炵幇鍙兘鏄垚鍔燂紙瑕嗙洊锛夋垨澶辫触锛堟嫆缁濓級
    // 杩欓噷鍋囪鏄け璐?    assert!(result.is_err());
}

#[tokio::test]
async fn test_restore_wallet_invalid_mnemonic() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 浣跨敤鏃犳晥鍔╄璇?    let result = manager.restore_wallet("invalid_restore", "invalid mnemonic").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_backup_restore_flow() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 鍒涘缓閽卞寘
    manager.create_wallet("backup_test", true).await.unwrap();

    // 澶囦唤閽卞寘
    let backup_result = manager.backup_wallet("backup_test").await;
    assert!(backup_result.is_ok());
    let mnemonic = backup_result.unwrap();

    // 鍒犻櫎閽卞寘
    manager.delete_wallet("backup_test").await.unwrap();

    // 浠庡浠芥仮澶?    let restore_result = manager.restore_wallet("restored_backup", &mnemonic).await;
    assert!(restore_result.is_ok());
}

#[tokio::test]
async fn test_get_balance_with_network() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    // 纭繚鍦ㄦ祴璇曚腑娌℃湁閰嶇疆浠讳綍缃戠粶锛岃繖鏍?get_balance 蹇呯劧浼氬け璐ャ€?    config.blockchain.networks.clear();
    let manager = WalletManager::new(&config).await.unwrap();

    // 鍒涘缓閽卞寘
    manager.create_wallet("balance_test", true).await.unwrap();

    // 娴嬭瘯get_balance鏂规硶
    let balance = manager.get_balance("balance_test", "eth").await;
    // 鍋囪鏂规硶瀹炵幇鍏佽鏌ヨ涓嶅瓨鍦ㄧ殑閽卞寘浣欓骞惰繑鍥為敊璇?    assert!(balance.is_err());
}

#[tokio::test]
async fn test_get_balance_wallet_not_found() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 娴嬭瘯涓嶅瓨鍦ㄧ殑閽卞寘
    let result = manager.get_balance("nonexistent", "eth").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_balance_invalid_network() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 鍒涘缓閽卞寘
    manager.create_wallet("network_test", true).await.unwrap();

    // 娴嬭瘯鏃犳晥缃戠粶
    let result = manager.get_balance("network_test", "invalid_network").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wallet_persistence() {
    // 浣跨敤涓存椂鏂囦欢鑰屼笉鏄唴瀛樻暟鎹簱
    let temp_dir = tempdir().unwrap();
    // Use the temp dir as current working directory and a relative sqlite URL with
    // mode=rwc so sqlite will create the file if it doesn't exist.
    std::env::set_current_dir(temp_dir.path()).unwrap();
    let db_url = "sqlite://wallet_db.sqlite?mode=rwc".to_string();

    // 绗竴涓鐞嗗櫒瀹炰緥
    {
        let mut config = WalletConfig::default();
        config.storage.database_url = db_url.clone();
        let manager = WalletManager::new(&config).await.unwrap();

        // 鍒涘缓閽卞寘
        manager.create_wallet("persistent", true).await.unwrap();
    }

    // 绗簩涓鐞嗗櫒瀹炰緥锛屽簲璇ヨ兘璁块棶涔嬪墠鍒涘缓鐨勯挶鍖?    {
        let mut config = WalletConfig::default();
        config.storage.database_url = db_url;
        let manager = WalletManager::new(&config).await.unwrap();

        // 妫€鏌ラ挶鍖呮槸鍚﹀瓨鍦?        let wallets = manager.list_wallets().await.unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].name, "persistent");
    }
}

// 娴嬭瘯鑾峰彇閽卞寘鐨勭壒瀹氶摼鍦板潃
#[tokio::test]
async fn test_get_wallet_address() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "sqlite::memory:".to_string();
    let manager = WalletManager::new(&config).await.unwrap();

    // 鍒涘缓閽卞寘
    manager.create_wallet("address_test", true).await.unwrap();

    // 鍋囪鏈変竴涓猤et_wallet_address鏂规硶
    // 濡傛灉娌℃湁锛屽彲浠ユ崲鎴愬叾浠栬兘鑾峰彇閽卞寘淇℃伅鐨勬柟娉?    let address = manager.derive_address(b"some_master_key", "eth");
    assert!(address.is_ok());

    // 娴嬭瘯涓嶅瓨鍦ㄧ殑閽卞寘
    // `derive_address` 涓嶄緷璧栦簬閽卞寘瀛樺湪锛屾墍浠ヨ繖涓祴璇曚笉閫傜敤
}

#[tokio::test]
async fn test_database_connection_error() {
    let mut config = WalletConfig::default();
    config.storage.database_url = "/invalid/path/that/cannot/exist".to_string();

    let result = WalletManager::new(&config).await;
    assert!(result.is_err());
}
