// filepath: src/blockchain/bridge/tests.rs
use super::mock::{EthereumToSolanaBridge, SolanaToEthereumBridge};
use super::transfer::transfer_assets;
use super::relay::relay_transaction;
use crate::blockchain::traits::Bridge;

#[tokio::test]
async fn test_mock_ethereum_to_solana_bridge() {
    let bridge = EthereumToSolanaBridge::new("0x...EthSolBridge...");
    let tx_hash = bridge.transfer("0xFrom", "SolTo", "1.0").await.unwrap();
    assert_eq!(tx_hash, "mock_eth_to_sol_tx_hash");

    let status = bridge.get_status("tx123").await.unwrap();
    assert_eq!(status, "completed");
}

#[tokio::test]
async fn test_mock_solana_to_ethereum_bridge() {
    let bridge = SolanaToEthereumBridge::new("0x...SolEthBridge...");
    let tx_hash = bridge.transfer("SolFrom", "0xTo", "1.0").await.unwrap();
    assert_eq!(tx_hash, "mock_sol_to_eth_tx_hash");

    let status = bridge.get_status("tx456").await.unwrap();
    assert_eq!(status, "completed");
}

#[tokio::test]
async fn test_transfer_assets_via_interface() {
    let bridge = EthereumToSolanaBridge::new("0x...EthSolBridge...");
    let tx_hash = transfer_assets(&bridge, "0xFrom", "SolTo", "1.0").await.unwrap();
    assert_eq!(tx_hash, "mock_eth_to_sol_tx_hash");
}

#[tokio::test]
async fn test_relay_transaction_via_interface() {
    let bridge = SolanaToEthereumBridge::new("0x...SolEthBridge...");
    let status = relay_transaction(&bridge, "tx789").await.unwrap();
    assert_eq!(status, "completed");
}