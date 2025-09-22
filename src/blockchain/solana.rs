use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info};

use super::traits::{BlockchainClient, TransactionStatus};

pub struct SolanaClient {
    _rpc_url: String,
    network_name: String,
}

impl SolanaClient {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        info!("🔗 Connecting to Solana network: {}", rpc_url);

        // Determine network name from RPC URL
        let network_name = if rpc_url.contains("mainnet") {
            "solana".to_string()
        } else if rpc_url.contains("devnet") {
            "solana-devnet".to_string()
        } else if rpc_url.contains("testnet") {
            "solana-testnet".to_string()
        } else {
            "solana-custom".to_string()
        };

        info!("✅ Connected to {} (simulated)", network_name);

        Ok(Self {
            _rpc_url: rpc_url.to_string(),
            network_name,
        })
    }

    // 简单静态校验：Base58 且 32 字节
    pub fn validate_solana_address(addr: &str) -> bool {
        match bs58::decode(addr).into_vec() {
            Ok(bytes) => bytes.len() == 32,
            Err(_) => false,
        }
    }
}

#[async_trait]
impl BlockchainClient for SolanaClient {
    async fn get_balance(&self, address: &str) -> Result<String> {
        debug!("Getting SOL balance for address: {}", address);

        if !SolanaClient::validate_solana_address(address) {
            return Err(anyhow::anyhow!("Invalid Solana address: {}", address));
        }

        // Simulated balance - in a real implementation, this would call the Solana RPC
        let balance_sol = "1.234567890";

        debug!("✅ Balance: {} SOL (simulated)", balance_sol);
        Ok(balance_sol.to_string())
    }

    async fn send_transaction(
        &self,
        private_key: &[u8],
        to_address: &str,
        amount: &str,
    ) -> Result<String> {
        info!("💸 Sending {} SOL to {} (simulated)", amount, to_address);

        if private_key.len() != 32 {
            return Err(anyhow::anyhow!("Private key must be 32 bytes for Solana"));
        }

        if !SolanaClient::validate_solana_address(to_address) {
            return Err(anyhow::anyhow!("Invalid recipient address: {}", to_address));
        }

        // Parse amount
        let _amount_f64: f64 = amount
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

        // Simulated transaction hash
        let tx_hash = format!("simulated_solana_tx_{}", chrono::Utc::now().timestamp());

        info!("✅ Transaction sent (simulated): {}", tx_hash);
        Ok(tx_hash)
    }

    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus> {
        debug!("Getting transaction status for: {} (simulated)", tx_hash);

        // Simulate confirmed status for transactions that look like ours
        if tx_hash.starts_with("simulated_solana_tx_") {
            Ok(TransactionStatus::Confirmed)
        } else {
            Ok(TransactionStatus::Unknown)
        }
    }

    async fn estimate_fee(&self, _to_address: &str, _amount: &str) -> Result<String> {
        debug!("Estimating Solana transaction fee (simulated)");

        // Solana typically has very low fees (around 0.000005 SOL)
        let fee_sol = "0.000005000";

        debug!("✅ Estimated fee: {} SOL (simulated)", fee_sol);
        Ok(fee_sol.to_string())
    }

    async fn get_block_number(&self) -> Result<u64> {
        // Simulate current slot number
        let slot = chrono::Utc::now().timestamp() as u64;
        Ok(slot)
    }

    fn validate_address(&self, address: &str) -> Result<bool> {
        Ok(SolanaClient::validate_solana_address(address))
    }

    fn get_network_name(&self) -> &str {
        &self.network_name
    }

    fn get_native_token(&self) -> &str {
        "SOL"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_validation() {
        // Valid Solana address (SystemProgram)
        assert!(SolanaClient::validate_solana_address(
            "11111111111111111111111111111111"
        ));

        // Invalid addresses
        assert!(!SolanaClient::validate_solana_address("invalid_address"));
        assert!(!SolanaClient::validate_solana_address(
            "0x742d35Cc6635C0532925a3b8D400e8B78fFe4860"
        )); // Ethereum format
    }

    #[tokio::test]
    async fn test_solana_client() {
        let client = SolanaClient::new("https://api.devnet.solana.com")
            .await
            .unwrap();

        assert_eq!(client.get_network_name(), "solana-devnet");
        assert_eq!(client.get_native_token(), "SOL");

        // Test address validation (using a valid-looking but not necessarily real address)
        assert!(client
            .validate_address("Vote111111111111111111111111111111111111111")
            .unwrap());
        assert!(!client.validate_address("invalid").unwrap());
    }
}
