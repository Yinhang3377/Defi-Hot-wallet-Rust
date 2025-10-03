// tests/bridge_tests.rs
//! 妗ユ帴鍔熻兘娴嬭瘯

use anyhow::Result;
use chrono::Utc;
use defi_hot_wallet::blockchain::bridge::{
    Bridge, BridgeTransactionStatus, EthereumToBSCBridge, EthereumToSolanaBridge,
    PolygonToEthereumBridge, SolanaToEthereumBridge,
};
use defi_hot_wallet::core::wallet_info::{SecureWalletData, WalletInfo};
use std::str::FromStr;
use uuid::Uuid;

fn create_mock_wallet_data() -> SecureWalletData {
    SecureWalletData {
        info: WalletInfo {
            id: Uuid::from_str("12345678-1234-1234-1234-123456789012").unwrap(),
            name: "test-wallet".to_string(),
            created_at: Utc::now(),
            quantum_safe: true,
            multi_sig_threshold: 1,
            networks: vec!["eth".to_string(), "solana".to_string()],
        },
        encrypted_master_key: vec![1, 2, 3, 4],
        salt: vec![5, 6, 7, 8],
        nonce: vec![9, 10, 11, 12],
    }
}

#[tokio::test]
async fn test_ethereum_to_solana_bridge() -> Result<()> {
    let bridge = EthereumToSolanaBridge::new("0xMockBridgeContract");
    let wallet_data = create_mock_wallet_data();

    let tx_hash =
        bridge.transfer_across_chains("eth", "solana", "USDC", "100.0", &wallet_data).await?;

    assert!(tx_hash.starts_with("0x_simulated_lock_tx_"));

    let status = bridge.check_transfer_status(&tx_hash).await?;
    assert!(
        matches!(status, BridgeTransactionStatus::InTransit | BridgeTransactionStatus::Completed),
        "Expected status to be InTransit or Completed, but got {:?}",
        status
    );
    Ok(())
}

#[tokio::test]
async fn test_solana_to_ethereum_bridge() -> Result<()> {
    let bridge = SolanaToEthereumBridge::new("0xMockReverseBridgeContract");
    let wallet_data = create_mock_wallet_data();

    bridge.transfer_across_chains("solana", "eth", "USDC", "50.0", &wallet_data).await?;
    Ok(())
}

#[tokio::test]
async fn test_ethereum_to_bsc_bridge() -> Result<()> {
    let bridge = EthereumToBSCBridge::new("0xMockEthBscBridge");
    let wallet_data = create_mock_wallet_data();

    bridge.transfer_across_chains("eth", "bsc", "USDT", "75.0", &wallet_data).await?;
    Ok(())
}

#[tokio::test]
async fn integration_transfer_and_failed_marker() -> Result<()> {
    let bridge = EthereumToSolanaBridge::new("0xBridge");
    let w = create_mock_wallet_data();

    let tx = bridge.transfer_across_chains("eth", "solana", "USDC", "1.0", &w).await?;
    assert!(tx.starts_with("0x_simulated_lock_tx_"));

    // explicit failed marker forces Failed status
    let failed_tx = "0x_marked_failed_tx";
    let status = bridge.check_transfer_status(failed_tx).await?;
    assert_eq!(
        status,
        BridgeTransactionStatus::Failed("Transaction explicitly marked as failed".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn integration_mock_bridge_variants_and_concurrent() -> Result<()> {
    let s2e = SolanaToEthereumBridge::new("0xS2E");
    let e2b = EthereumToBSCBridge::new("0xE2B");
    let poly = PolygonToEthereumBridge::new("0xP2E");
    let w = create_mock_wallet_data();

    let t1 = s2e.transfer_across_chains("solana", "eth", "USDC", "1.0", &w).await?;
    assert!(t1.starts_with("0x_simulated_tx_"));

    let t2 = e2b.transfer_across_chains("eth", "bsc", "USDT", "2.0", &w).await?;
    assert!(t2.starts_with("0x_simulated_tx_"));

    let t3 = poly.transfer_across_chains("polygon", "eth", "DAI", "3.0", &w).await?;
    assert!(t3.starts_with("0x_simulated_tx_"));

    // concurrent transfers should all succeed
    let handles = vec![
        tokio::spawn({
            let s2e = SolanaToEthereumBridge::new("0xS2E");
            let w = create_mock_wallet_data();
            async move { s2e.transfer_across_chains("solana", "eth", "USDC", "1.0", &w).await }
        }),
        tokio::spawn({
            let e2b = EthereumToBSCBridge::new("0xE2B");
            let w = create_mock_wallet_data();
            async move { e2b.transfer_across_chains("eth", "bsc", "USDT", "2.0", &w).await }
        }),
    ];

    let results = futures::future::join_all(handles).await;
    for r in results {
        let ok = r.expect("task panicked")?;
        assert!(ok.starts_with("0x_simulated_tx_"));
    }

    Ok(())
}
