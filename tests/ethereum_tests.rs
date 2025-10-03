use defi_hot_wallet::blockchain::ethereum::EthereumClient;
use defi_hot_wallet::blockchain::traits::BlockchainClient;
use ethers::providers::{Http, Provider};
use std::convert::TryFrom;

#[tokio::test(flavor = "current_thread")]
async fn send_transaction_invalid_key_errors() {
    let provider = Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap();
    let client = EthereumClient::new_with_provider(provider);
    let short_key = [0u8; 16];
    let res = client
        .send_transaction(&short_key, "0x0000000000000000000000000000000000000000", "0.01")
        .await;
    assert!(res.is_err());
}

#[test]
fn validate_address_public_api() {
    // 涓轰簡璋冪敤 validate_address锛屾垜浠渶瑕佷竴涓?EthereumClient 鐨勫疄渚?    let provider = Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap();
    let client = EthereumClient::new_with_provider(provider);

    assert!(client.validate_address("0x742d35Cc6634C0532925a3b8D400e8B78fFe4860").unwrap());
    assert!(!client.validate_address("abc").unwrap());
}
