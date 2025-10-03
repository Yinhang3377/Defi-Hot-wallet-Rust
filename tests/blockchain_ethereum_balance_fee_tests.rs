//! tests/blockchain_ethereum_balance_fee_tests.rs
//!
//! Tests for Ethereum blockchain client balance and fee estimation functionality.

use defi_hot_wallet::blockchain::ethereum::*;
use defi_hot_wallet::blockchain::traits::BlockchainClient; // 瀵煎叆 BlockchainClient trait
use ethers::providers::{MockProvider, Provider};
use ethers::types::U256;

fn create_mock_client() -> (EthereumClient<MockProvider>, MockProvider) {
    let mock_provider = MockProvider::new();
    let provider = Provider::new(mock_provider.clone());
    (EthereumClient::new_with_provider(provider), mock_provider)
}

#[tokio::test]
async fn test_get_balance_valid_address() {
    let (client, mock_provider) = create_mock_client();
    let address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";

    // 閰嶇疆 MockProvider 鐨勫搷搴?    let _ = mock_provider.push(U256::from(100_000_000_000_000_000u64)); // 妯℃嫙杩斿洖浣欓 0.1 ether (10^17 wei)

    let balance = client.get_balance(address).await.unwrap();
    assert_eq!(balance, "0.100000000000000000"); // 璋冩暣鏂█鍊硷紝鍖归厤瀹為檯杩斿洖鍊?}

#[tokio::test]
async fn test_get_balance_invalid_address() {
    let (client, _) = create_mock_client();
    let result = client.get_balance("invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_balance_empty_address() {
    let (client, _) = create_mock_client();
    let result = client.get_balance("").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_estimate_fee_valid_inputs() {
    let (client, mock_provider) = create_mock_client();
    let to_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let amount = "0.1";

    // 閰嶇疆 MockProvider 鐨勫搷搴?    let _ = mock_provider.push(U256::from(42_000_000_000u64)); // 妯℃嫙杩斿洖 gas price 42 Gwei (42 * 10^9 wei)
    let _ = mock_provider.push(U256::from(21_000u64)); // 妯℃嫙杩斿洖 gas limit 21000

    let fee = client.estimate_fee(to_address, amount).await.unwrap();
    assert_eq!(fee, "0.000000000441000000"); // 璋冩暣鏂█鍊硷紝鍖归厤浠ュお鍗曚綅
}

#[tokio::test]
async fn test_estimate_fee_invalid_to_address() {
    let (client, _) = create_mock_client();
    let result = client.estimate_fee("invalid", "0.1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_estimate_fee_invalid_amount() {
    let (client, _) = create_mock_client();
    let to_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let result = client.estimate_fee(to_address, "invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_estimate_fee_empty_to_address() {
    let (client, _) = create_mock_client();
    let result = client.estimate_fee("", "0.1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_estimate_fee_empty_amount() {
    let (client, _) = create_mock_client();
    let to_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let result = client.estimate_fee(to_address, "").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_estimate_fee_negative_amount() {
    let (client, _) = create_mock_client();
    let to_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let result = client.estimate_fee(to_address, "-0.1").await;
    assert!(result.is_err());
}
