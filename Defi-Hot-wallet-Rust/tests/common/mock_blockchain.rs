use defi_hot_wallet::blockchain::BlockchainClient;

struct MockBlockchainClient;

impl BlockchainClient for MockBlockchainClient {
    fn validate_address(&self, _address: &str) -> Result<bool, String> {
        Ok(true)
    }

    async fn get_transaction_status(&self, _tx_hash: &str) -> Result<String, String> {
        Ok("Success".to_string())
    }

    async fn estimate_fee(&self, _to_address: &str, _amount: &str) -> Result<String, String> {
        Ok("0.01".to_string())
    }

    async fn get_balance(&self, _address: &str) -> Result<String, String> {
        Ok("100".to_string())
    }

    async fn send_transaction(&self, _private_key: &[u8], _to_address: &str, _amount: &str) -> Result<String, String> {
        Ok("tx_hash".to_string())
    }
}