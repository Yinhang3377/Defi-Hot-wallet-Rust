// tests/bridge_tests.rs
//! 桥接功能测试

use anyhow::Result;
use chrono::Utc;
use defi_hot_wallet::blockchain::bridge::{
    Bridge, BridgeTransactionStatus, EthereumToBSCBridge, EthereumToSolanaBridge,
    SolanaToEthereumBridge,
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
