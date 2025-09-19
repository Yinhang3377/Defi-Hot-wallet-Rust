use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BlockchainClient: Send + Sync {
    /// Get the balance of an address
    async fn get_balance(&self, address: &str) -> Result<String>;
    
    /// Send a transaction
    async fn send_transaction(
        &self,
        private_key: &[u8],
        to_address: &str,
        amount: &str,
    ) -> Result<String>;
    
    /// Get transaction status
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus>;
    
    /// Estimate transaction fee
    async fn estimate_fee(&self, to_address: &str, amount: &str) -> Result<String>;
    
    /// Get current block number
    async fn get_block_number(&self) -> Result<u64>;
    
    /// Validate an address
    fn validate_address(&self, address: &str) -> Result<bool>;
    
    /// Get network name
    fn get_network_name(&self) -> &str;
    
    /// Get native token symbol
    fn get_native_token(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub fee: String,
    pub block_number: Option<u64>,
    pub confirmations: u64,
    pub status: TransactionStatus,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}