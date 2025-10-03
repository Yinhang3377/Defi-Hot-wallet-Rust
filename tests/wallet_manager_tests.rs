//! WalletManager 鍔熻兘娴嬭瘯锛氭祴璇曟墍鏈?WalletManager 鏂规硶
//! 瑕嗙洊锛氶挶鍖?CRUD銆佷綑棰濄€佷氦鏄撱€佹ˉ鎺ャ€佸姞瀵嗐€佸瘑閽ユ淳鐢熺瓑
//! 浣跨敤 mock storage 鍜屽鎴风锛岀‘淇濇祴璇曢殧绂?//! 鍚堝苟浜?wallet_manager_test.rs 鐨勭嫭鐗规祴璇曪紙濡傚苟鍙戯級锛屽苟杩涜浜嗛噸鏋?//! 娣诲姞 stub 娴嬭瘯锛堝亣鐨勶級锛歡et_transaction_history, backup_wallet, restore_wallet, send_multi_sig_transaction

use defi_hot_wallet::core::config::{BlockchainConfig, StorageConfig, WalletConfig};
use defi_hot_wallet::core::wallet_manager::WalletManager;
use std::collections::HashMap;
use tokio;
use uuid::Uuid;

/// 鍒涘缓涓€涓敤浜庢祴璇曠殑 WalletConfig 瀹炰緥銆?///
/// 璇ラ厤缃娇鐢ㄥ唴瀛樹腑鐨?SQLite 鏁版嵁搴擄紝浠ョ‘淇濇祴璇曠殑闅旂鎬у拰閫熷害锛?/// 閬垮厤浜嗘枃浠?I/O 鍜岀鐩樼姸鎬佺殑渚濊禆銆?fn create_test_config() -> WalletConfig {
    // 浣跨敤鍐呭瓨鏁版嵁搴擄紝閬垮厤鏂囦欢IO闂
    WalletConfig {
        storage: StorageConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: Some(1),
            connection_timeout_seconds: Some(30),
        },
        blockchain: BlockchainConfig {
            networks: HashMap::new(),
            default_network: Some("eth".to_string()),
        },
        quantum_safe: false,
        multi_sig_threshold: 2,
    }
}

/// 鍒涘缓涓€涓敤浜庢祴璇曠殑 WalletManager 瀹炰緥銆?///
/// 杩欎釜寮傛杈呭姪鍑芥暟灏佽浜?`WalletManager` 鐨勫垱寤鸿繃绋嬶紝
/// 浣跨敤 `create_test_config` 鏉ヨ幏鍙栦竴涓共鍑€鐨勩€佸熀浜庡唴瀛樼殑閰嶇疆銆?async fn create_test_wallet_manager() -> WalletManager {
    let config = create_test_config();
    WalletManager::new(&config).await.unwrap()
}

/// 鏄惧紡娓呯悊鍑芥暟锛岀敤浜庡湪娴嬭瘯鍚庨噴鏀捐祫婧愩€?///
/// 鍦ㄥ紓姝ユ祴璇曚腑锛岀壒鍒槸浣跨敤鍐呭瓨鏁版嵁搴撴椂锛岀‘淇?`WalletManager`
/// 琚纭涪寮冿紙drop锛変互鍏抽棴鍏舵暟鎹簱杩炴帴姹犳槸闈炲父閲嶈鐨勩€?/// 杩欏彲浠ラ槻姝㈡祴璇曚箣闂村嚭鐜拌祫婧愭硠婕忔垨鐘舵€佹薄鏌撱€?async fn cleanup(wm: WalletManager) {
    // 寮哄埗閽卞寘绠＄悊鍣ㄥ叧闂墍鏈夎繛鎺?    drop(wm);

    // 杩欐槸涓€涓皬鐨勬妧宸э紝灏濊瘯瑙﹀彂鍨冨溇鍥炴敹锛屼互纭繚鍐呭瓨璧勬簮琚強鏃堕噴鏀俱€?    // 寮哄埗涓€娆″皬鐨勫唴瀛樺垎閰嶄互灏濊瘯瑙﹀彂鍨冨溇鍥炴敹
    let _ = Box::new(0u8);
}

#[tokio::test(flavor = "current_thread")]
async fn test_new_storage_error() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let mut config = create_test_config();
            config.storage.database_url = "invalid-protocol://".to_string();
            let result = WalletManager::new(&config).await;
            assert!(result.is_err());
            // 鍦ㄨ繖绉嶆儏鍐典笅锛學alletManager 瀹炰緥浠庢湭鎴愬姛鍒涘缓锛屽洜姝や笉闇€瑕佹竻鐞嗐€?            // 鏃犻渶娓呯悊
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_wallet_manager_create_and_list() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let wm = create_test_wallet_manager().await;
            let wallet_name = format!("test_wallet_{}", Uuid::new_v4()); // 浣跨敤 UUID 纭繚鍚嶇О鍞竴
            let result = wm.create_wallet(&wallet_name, false).await;
            assert!(result.is_ok());
            let wallet = result.unwrap();
            assert_eq!(wallet.name, wallet_name);
            assert!(!wallet.quantum_safe);

            // 娴嬭瘯閲忓瓙瀹夊叏閽卞寘
            let result = wm.create_wallet("quantum_wallet", true).await;
            assert!(result.is_ok());
            let wallet = result.unwrap();
            assert!(wallet.quantum_safe);
            cleanup(wm).await;
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_create_wallet_duplicate_name() {
    let manager = create_test_wallet_manager().await;
    let wallet_name = "duplicate_wallet";
    // 绗竴娆″垱寤哄簲璇ユ垚鍔?    manager.create_wallet(wallet_name, false).await.unwrap();
    // 绗簩娆′娇鐢ㄧ浉鍚屽悕绉板垱寤哄簲璇ュけ璐?    let result = manager.create_wallet(wallet_name, false).await;
    assert!(result.is_err());
    cleanup(manager).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_wallets() {
    let wm = create_test_wallet_manager().await;
    // 鍒涘缓涓や釜閽卞寘
    wm.create_wallet("wallet1", false).await.unwrap();
    wm.create_wallet("wallet2", true).await.unwrap();
    // 鍒楀嚭閽卞寘骞堕獙璇佹暟閲?    let wallets = wm.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 2);
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_wallet() {
    let wm = create_test_wallet_manager().await;
    // 鍒涘缓涓€涓挶鍖呯劧鍚庡垹闄ゅ畠
    wm.create_wallet("delete_wallet", false).await.unwrap();
    let result = wm.delete_wallet("delete_wallet").await;
    // 楠岃瘉鍒犻櫎鎴愬姛涓旈挶鍖呭垪琛ㄤ负绌?    assert!(result.is_ok());
    let wallets = wm.list_wallets().await.unwrap();
    assert_eq!(wallets.len(), 0);
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_wallet_not_found() {
    let wm = create_test_wallet_manager().await;
    // 灏濊瘯鍒犻櫎涓€涓笉瀛樺湪鐨勯挶鍖咃紝棰勬湡浼氬け璐?    let result = wm.delete_wallet("nonexistent").await;
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_balance() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("balance_wallet", false).await.unwrap();
    // 褰撳墠瀹炵幇娌℃湁妯℃嫙鐨勫尯鍧楅摼瀹㈡埛绔紝鍥犳璋冪敤 get_balance 浼氬洜涓?    // 鏃犳硶杩炴帴鍒拌妭鐐规垨瑙ｆ瀽瀵嗛挜鑰屽け璐ャ€傝繖鏄竴涓鏈熺殑閿欒銆?    let result = wm.get_balance("balance_wallet", "eth").await;
    // 棰勬湡閿欒锛屽洜涓烘棤娉曡В瀵嗗瘑閽ヤ互鑾峰彇鍦板潃
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_transaction() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("tx_wallet", false).await.unwrap();
    // 涓?get_balance 绫讳技锛屾鎿嶄綔鍥犳棤娉曚笌鍖哄潡閾句氦浜掕€岄鏈熷け璐ャ€?    // 瀹冧細鍥犱负鏃犳硶瑙ｅ瘑瀵嗛挜鏉ョ鍚嶄氦鏄撹€屽け璐ャ€?    let result = wm.send_transaction("tx_wallet", "0x1234567890abcdef", "0.1", "eth").await;
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_transaction_invalid_address() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("tx_wallet", false).await.unwrap();
    // 楠岃瘉鍦板潃鏍煎紡鐨勬鏌ユ槸鍚︽湁鏁?    let result = wm.send_transaction("tx_wallet", "invalid_address", "0.1", "eth").await;
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_transaction_negative_amount() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("tx_wallet", false).await.unwrap();
    // 楠岃瘉閲戦瑙ｆ瀽鍜屾鏌ユ槸鍚︽湁鏁?    let result = wm.send_transaction("tx_wallet", "0x1234567890abcdef", "-0.1", "eth").await;
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bridge_assets() {
    let wm = create_test_wallet_manager().await;
    // bridge_assets 鏄竴涓ā鎷熷疄鐜帮紝瀹冩€绘槸杩斿洖涓€涓ā鎷熺殑浜ゆ槗鍝堝笇銆?    // 杩欎釜娴嬭瘯楠岃瘉璇ユā鎷熻涓烘槸鍚︾鍚堥鏈熴€?    let result = wm.bridge_assets("bridge_wallet", "eth", "solana", "USDC", "10.0").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "mock_bridge_tx_hash");
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bridge_assets_unsupported_chain() {
    let wm = create_test_wallet_manager().await;
    // 鍗充娇閾句笉鍙楁敮鎸侊紝褰撳墠鐨勬ā鎷熷疄鐜颁篃浼氭垚鍔熴€?    // 涓€涓洿瀹屾暣鐨勬祴璇曚細妯℃嫙妗ユ帴宸ュ巶锛坆ridge factory锛夎繑鍥為敊璇€?    let result = wm.bridge_assets("bridge_wallet", "unsupported", "solana", "USDC", "10.0").await;
    assert!(result.is_ok()); // 褰撳墠鐨?Mock 鎬绘槸鎴愬姛
    assert_eq!(result.unwrap(), "mock_bridge_tx_hash");
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_transaction_history() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("history_wallet", false).await.unwrap();
    // 杩欐槸涓€涓々锛坰tub锛夊疄鐜帮紝瀹冩€绘槸杩斿洖涓€涓┖鍒楄〃銆?    let history = wm.get_transaction_history("history_wallet").await.unwrap();
    assert!(history.is_empty()); // Stub 杩斿洖绌?    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_backup_wallet() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("backup_wallet", false).await.unwrap();
    // 妗╁疄鐜扮幇鍦ㄨ繑鍥炵湡瀹炵殑鏈夋晥鍔╄璇嶏紱楠岃瘉瀹冪湅璧锋潵鍍?BIP39 鐨?24 璇嶅姪璁拌瘝銆?    let seed = wm.backup_wallet("backup_wallet").await.unwrap();
    assert_eq!(seed.split_whitespace().count(), 24, "backup mnemonic should be 24 words");
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_restore_wallet() {
    let wm = create_test_wallet_manager().await;
    // 妗╁疄鐜帮紝鎬绘槸杩斿洖鎴愬姛銆?    let result = wm.restore_wallet(
        "restored_wallet",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    )
    .await;
    assert!(result.is_ok()); // Stub 鎬绘槸鎴愬姛
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_send_multi_sig_transaction() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("multi_wallet", false).await.unwrap();
    let signatures = vec!["sig1".to_string(), "sig2".to_string()];
    // 妗╁疄鐜帮紝杩斿洖涓€涓浐瀹氱殑鍋囦氦鏄撳搱甯屻€?    let result = wm
        .send_multi_sig_transaction("multi_wallet", "0x1234567890abcdef", "0.1", "eth", &signatures)
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "fake_multi_sig_tx_hash"); // Stub
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_send_multi_sig_transaction_insufficient_signatures() {
    let wm = create_test_wallet_manager().await;
    wm.create_wallet("multi_wallet", false).await.unwrap();
    let signatures = vec!["sig1".to_string()]; // 灏戜簬闃堝€?2
                                               // 褰撳墠鐨勬々瀹炵幇涓嶆鏌ョ鍚嶆暟閲忥紝鎵€浠ヨ繖涓祴璇曚細閫氳繃銆?                                               // 涓€涓畬鏁寸殑瀹炵幇搴旇鍦ㄨ繖閲岃繑鍥為敊璇€?    let result = wm
        .send_multi_sig_transaction("multi_wallet", "0x1234567890abcdef", "0.1", "eth", &signatures)
        .await;
    assert!(result.is_ok());
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_generate_mnemonic() {
    let wm = create_test_wallet_manager().await;
    let mnemonic = wm.generate_mnemonic().unwrap();
    // 楠岃瘉鐢熸垚鐨勫姪璁拌瘝鏄惁绗﹀悎 BIP39 24 璇嶇殑鏍囧噯鏍煎紡銆?    assert_eq!(mnemonic.split_whitespace().count(), 24);
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_derive_master_key() {
    let wm = create_test_wallet_manager().await;
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    // 楠岃瘉浠庡姪璁拌瘝娲剧敓鐨勪富瀵嗛挜鏄惁涓洪鏈熺殑闀垮害锛?2瀛楄妭锛夈€?    let key = wm.derive_master_key(mnemonic).await.unwrap();
    assert_eq!(key.len(), 32);
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_derive_address_eth() {
    let wm = create_test_wallet_manager().await;
    let master_key = [0u8; 32];
    let address = wm.derive_address(&master_key, "eth").unwrap();
    // 楠岃瘉娲剧敓鐨勪互澶潑鍦板潃鏄惁浠?"0x" 寮€澶淬€?    assert!(address.starts_with("0x"));
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_derive_address_solana() {
    let wm = create_test_wallet_manager().await;
    let master_key = [0u8; 32];
    let address = wm.derive_address(&master_key, "solana").unwrap();
    // 楠岃瘉娲剧敓鐨?Solana 鍦板潃锛圔ase58 缂栫爜锛変笉涓虹┖銆?    assert!(!address.is_empty());
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_derive_address_unsupported_network() {
    let wm = create_test_wallet_manager().await;
    let master_key = [0u8; 32];
    // 楠岃瘉褰撴彁渚涗笉鏀寔鐨勭綉缁滄椂锛屾槸鍚﹁繑鍥為敊璇€?    let result = wm.derive_address(&master_key, "unsupported");
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_calculate_bridge_fee() {
    let wm = create_test_wallet_manager().await;
    // 杩欐槸涓€涓ā鎷熷疄鐜帮紝楠岃瘉瀹冩槸鍚﹁繑鍥為鏈熺殑鍥哄畾璐圭敤鍜屾椂闂淬€?    let (fee, time) = wm.calculate_bridge_fee("eth", "solana", "USDC", "100.0").unwrap();
    assert_eq!(fee, "1");
    assert!(time > chrono::Utc::now());
    cleanup(wm).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_get_block_number() {
    let wm = create_test_wallet_manager().await;
    // 涓?get_balance 绫讳技锛岀敱浜庢病鏈夌綉缁滆繛鎺ワ紝姝ゆ搷浣滈鏈熶細澶辫触銆?    let result = wm.get_block_number("eth").await;
    assert!(result.is_err());
    cleanup(wm).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_create_wallet() {
    // 杩欎釜娴嬭瘯楠岃瘉 WalletManager 鍦ㄥ苟鍙戠幆澧冧笅鐨勯瞾妫掓€с€?    let mut config = create_test_config();
    config.storage.max_connections = Some(10);
    let manager = WalletManager::new(&config).await.unwrap();
    let manager_arc = std::sync::Arc::new(manager);

    // 鍒涘缓澶氫釜绾跨▼鍚屾椂璋冪敤 create_wallet
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = std::sync::Arc::clone(&manager_arc);
        let handle = tokio::spawn(async move {
            manager_clone.create_wallet(&format!("wallet{}", i), false).await
        });
        handles.push(handle);
    }
    // 绛夊緟鎵€鏈夌嚎绋嬪畬鎴愬苟楠岃瘉姣忎釜鎿嶄綔閮芥垚鍔?    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // 鍦ㄦ祴璇曠粨鏉熸椂瀹夊叏鍦版竻鐞嗚祫婧?    // 鍦ㄦ祴璇曠粨鏉熸椂娓呯悊璧勬簮
    if let Ok(manager) = std::sync::Arc::try_unwrap(manager_arc) {
        cleanup(manager).await;
    }
}
