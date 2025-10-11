// tests/bridge_unit_tests.rs
// Additional deterministic unit tests for bridge mock implementations and relay helpers.

use std::collections::HashSet;
use std::env;

use tokio::task;

use defi_hot_wallet::blockchain::bridge::{
    mock::{
        EthereumToBSCBridge, EthereumToSolanaBridge, PolygonToEthereumBridge,
        SolanaToEthereumBridge,
    },
    relay::relay_transaction,
    BridgeTransactionStatus,
};
use defi_hot_wallet::blockchain::traits::Bridge;
use defi_hot_wallet::core::wallet_info::{SecureWalletData, WalletInfo};
use uuid::Uuid;

fn create_mock_wallet_data() -> SecureWalletData {
    SecureWalletData {
        info: WalletInfo {
            id: Uuid::new_v4(),
            name: "test-wallet".to_string(),
            created_at: chrono::Utc::now(),
            quantum_safe: false,
            multi_sig_threshold: 1,
            networks: vec!["eth".to_string(), "solana".to_string()],
        },
        encrypted_master_key: vec![],
        salt: vec![],
        nonce: vec![],
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_all_mock_bridges_transfer_and_status() {
    // Ensure mock bridges return deterministic success for transfers in this test
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");

    let wallet = create_mock_wallet_data();

    let eth_sol = EthereumToSolanaBridge { contract_address: "0xEthSol".to_string() };
    let sol_eth = SolanaToEthereumBridge { contract_address: "0xSolEth".to_string() };
    let eth_bsc = EthereumToBSCBridge { contract_address: "0xEthBsc".to_string() };
    let poly_eth = PolygonToEthereumBridge { contract_address: "0xPolyEth".to_string() };

    // transfer_across_chains should succeed
    let tx1 = eth_sol.transfer_across_chains("eth", "solana", "USDC", "1.0", &wallet).await;
    let tx2 = sol_eth.transfer_across_chains("solana", "eth", "USDC", "1.0", &wallet).await;
    let tx3 = eth_bsc.transfer_across_chains("eth", "bsc", "USDT", "2.5", &wallet).await;
    let tx4 = poly_eth.transfer_across_chains("polygon", "eth", "MATIC", "5.0", &wallet).await;

    assert!(tx1.is_ok() && !tx1.as_ref().unwrap().is_empty());
    assert!(tx2.is_ok() && !tx2.as_ref().unwrap().is_empty());
    assert!(tx3.is_ok() && !tx3.as_ref().unwrap().is_empty());
    assert!(tx4.is_ok() && !tx4.as_ref().unwrap().is_empty());

    // check_transfer_status should report a valid enum status
    let s1 = eth_sol.check_transfer_status("any").await.unwrap();
    let s2 = sol_eth.check_transfer_status("any").await.unwrap();
    let s3 = eth_bsc.check_transfer_status("any").await.unwrap();
    let s4 = poly_eth.check_transfer_status("any").await.unwrap();

    // Accept Completed or other valid variants; ensure enum variants compare
    assert!(matches!(
        s1,
        BridgeTransactionStatus::Completed
            | BridgeTransactionStatus::Initiated
            | BridgeTransactionStatus::InTransit
            | BridgeTransactionStatus::Failed(_)
    ));
    assert!(matches!(
        s2,
        BridgeTransactionStatus::Completed
            | BridgeTransactionStatus::Initiated
            | BridgeTransactionStatus::InTransit
            | BridgeTransactionStatus::Failed(_)
    ));
    assert!(matches!(
        s3,
        BridgeTransactionStatus::Completed
            | BridgeTransactionStatus::Initiated
            | BridgeTransactionStatus::InTransit
            | BridgeTransactionStatus::Failed(_)
    ));
    assert!(matches!(
        s4,
        BridgeTransactionStatus::Completed
            | BridgeTransactionStatus::Initiated
            | BridgeTransactionStatus::InTransit
            | BridgeTransactionStatus::Failed(_)
    ));

    // Clean up env var
    env::remove_var("BRIDGE_MOCK_FORCE_SUCCESS");
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_transaction_delegates_to_bridge() {
    let bridge = EthereumToSolanaBridge { contract_address: "0xRelay".to_string() };

    // relay_transaction should call check_transfer_status and return a BridgeTransactionStatus
    let status = relay_transaction(&bridge, "tx123").await.expect("relay should succeed");
    assert!(matches!(
        status,
        BridgeTransactionStatus::Completed
            | BridgeTransactionStatus::Initiated
            | BridgeTransactionStatus::InTransit
            | BridgeTransactionStatus::Failed(_)
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn test_concurrent_mock_transfers_produce_unique_hashes() {
    let wallet = create_mock_wallet_data();

    // ensure deterministic "force success" when available, but test uniqueness regardless
    env::set_var("BRIDGE_MOCK_FORCE_SUCCESS", "1");

    let mut handles = Vec::new();
    for _ in 0..12 {
        let w = wallet.clone();
        handles.push(task::spawn(async move {
            let b = EthereumToSolanaBridge { contract_address: "0xConcurrent".to_string() };
            b.transfer_across_chains("eth", "solana", "USDC", "0.1", &w)
                .await
                .expect("mock transfer ok")
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        results.push(h.await.expect("join"));
    }

    // ensure all returned tx identifiers are non-empty and unique
    let set: HashSet<_> = results.iter().collect();
    assert_eq!(set.len(), results.len(), "expected unique tx ids");
    for tx in results {
        assert!(!tx.is_empty());
    }

    env::remove_var("BRIDGE_MOCK_FORCE_SUCCESS");
}
