// ...existing code...
use anyhow::Result;
use async_trait::async_trait;
use ethers::{
    prelude::{JsonRpcClient, *},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
    utils::parse_ether,
};
use std::{str::FromStr, time::Duration};
use tracing::{debug, info, warn};

use super::traits::{BlockchainClient, TransactionStatus};

#[derive(Clone)]
pub struct EthereumClient<P: JsonRpcClient + Clone = Http> {
    provider: Provider<P>,
    network_name: String,
    chain_id: u64,
}

impl EthereumClient<Http> {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        // Clean RPC URL
        let rpc_url_clean = rpc_url.trim();
        let parsed_url = reqwest::Url::parse(rpc_url_clean).map_err(|e| {
            anyhow::anyhow!(
                "Invalid Ethereum RPC URL '{}': {}. Please check config.toml or env vars.",
                rpc_url_clean,
                e
            )
        })?;

        info!("Connecting to Ethereum network: {}", parsed_url);
        // Build a reqwest client with a short timeout; allow proxy environment vars.
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(10));
        if let Ok(proxy) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("HTTP_PROXY")) {
            if let Ok(p) = reqwest::Proxy::all(proxy) {
                builder = builder.proxy(p);
            }
        }
        let client =
            builder.build().map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

        let provider = Provider::new(Http::new_with_client(parsed_url.clone(), client));

        let chain_id = provider
            .get_chainid()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to get chain ID from {}. Error: {}. This might be due to a network issue, firewall, or an invalid RPC URL.", parsed_url, e)
            })?
            .as_u64();

        let network_name = match chain_id {
            1 => "ethereum".to_string(),
            11155111 => "sepolia".to_string(),
            137 => "polygon".to_string(),
            56 => "bsc".to_string(),
            97 => "bsctestnet".to_string(),
            _ => format!("ethereum-{}", chain_id),
        };

        info!("Connected to {} (Chain ID: {})", network_name, chain_id);

        Ok(Self { provider, network_name, chain_id })
    }

    pub async fn new_with_chain_id(rpc_url: &str, chain_id: u64) -> Result<Self> {
        info!("Connecting to Ethereum network: {} (Chain ID: {})", rpc_url, chain_id);

        // Reuse new() to build provider/client then override chain_id & network name.
        let temp_client = Self::new(rpc_url).await?;
        let provider = temp_client.provider;

        let network_name = match chain_id {
            1 => "ethereum".to_string(),
            11155111 => "sepolia".to_string(),
            137 => "polygon".to_string(),
            56 => "bsc".to_string(),
            97 => "bsctestnet".to_string(),
            _ => format!("ethereum-{}", chain_id),
        };

        info!("Connected to {} (Chain ID: {})", network_name, chain_id);

        Ok(Self { provider, network_name, chain_id })
    }
}

impl<P: JsonRpcClient + Clone> EthereumClient<P>
where
    // The `ethers` `Provider` requires its client `P` to be `Send + Sync` for async operations.
    // This bound is necessary for the `BlockchainClient` trait methods to be callable.
    P: Send + Sync,
{
    /// Creates a new EthereumClient with a given provider.
    /// This is useful for testing with a `MockProvider`.
    pub fn new_with_provider(provider: Provider<P>) -> EthereumClient<P> {
        EthereumClient {
            provider,
            network_name: "test".to_string(), // Default network name for testing
            chain_id: 1,                      // Default chain ID for testing (Ethereum Mainnet)
        }
    }

    fn create_wallet_from_private_key(&self, private_key: &[u8]) -> Result<LocalWallet> {
        // Debug: print to stderr so test runs without initializing tracing still show the info.
        eprintln!(
            "create_wallet_from_private_key: incoming private_key.len() = {}",
            private_key.len()
        );
        eprintln!(
            "create_wallet_from_private_key: bytes = {}",
            hex::encode(&private_key[..std::cmp::min(32, private_key.len())])
        );
        if private_key.len() != 32 {
            return Err(anyhow::anyhow!("Private key must be 32 bytes"));
        }

        let wallet = LocalWallet::from_bytes(private_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?
            .with_chain_id(self.chain_id);

        Ok(wallet)
    }

    pub async fn get_gas_price(&self) -> Result<U256> {
        eprintln!("get_gas_price: called");
        let res = self.provider.get_gas_price().await;
        match res {
            Ok(v) => {
                eprintln!("get_gas_price: got = 0x{:x}", v);
                Ok(v)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get gas price: {}", e)),
        }
    }

    pub async fn get_nonce(&self, address: &Address) -> Result<U256> {
        eprintln!("get_nonce: called for address: 0x{}", hex::encode(address));
        let res = self.provider.get_transaction_count(*address, None).await;
        match res {
            Ok(v) => {
                eprintln!("get_nonce: got = 0x{:x}", v);
                Ok(v)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get nonce: {}", e)),
        }
    }
}

#[async_trait]
impl<P> BlockchainClient for EthereumClient<P>
where
    P: JsonRpcClient + Clone + 'static + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn BlockchainClient> {
        Box::new(self.clone())
    }

    async fn get_balance(&self, address: &str) -> Result<String> {
        debug!("Getting ETH balance for address: {}", address);

        let address = Address::from_str(address)
            .map_err(|e| anyhow::anyhow!("Invalid Ethereum address: {}", e))?;

        let balance = self
            .provider
            .get_balance(address, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;

        let balance_eth = ethers::utils::format_ether(balance);
        debug!("Balance: {} ETH", balance_eth);

        Ok(balance_eth)
    }

    async fn send_transaction(
        &self,
        private_key: &[u8],
        to_address: &str,
        amount: &str,
    ) -> Result<String> {
        info!("Sending {} ETH to {}", amount, to_address);

        // Create wallet from private key
        let wallet = self
            .create_wallet_from_private_key(private_key)
            .map_err(|e| anyhow::anyhow!("Failed to create wallet from private key: {}", e))?;

        // Parse addresses and amount
        let to_address = Address::from_str(to_address)
            .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;

        let amount_wei =
            parse_ether(amount).map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

        // Get current gas price and nonce
        let gas_price = self.get_gas_price().await?;
        let nonce = self.get_nonce(&wallet.address()).await?;
        eprintln!("send_transaction: gas_price = 0x{:x}", gas_price);
        eprintln!("send_transaction: nonce = 0x{:x}", nonce);

        // Create transaction
        let tx = TransactionRequest::new()
            .to(to_address)
            .value(amount_wei)
            .gas_price(gas_price)
            .gas(21000u64) // Standard ETH transfer gas limit
            .nonce(nonce);

        // Sign and send transaction
        let client = SignerMiddleware::new(self.provider.clone(), wallet);

        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());

        info!("Transaction sent: {}", tx_hash);
        Ok(tx_hash)
    }

    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus> {
        debug!("Getting transaction status for: {}", tx_hash);

        let tx_hash = H256::from_str(tx_hash)
            .map_err(|e| anyhow::anyhow!("Invalid transaction hash: {}", e))?;

        match self.provider.get_transaction_receipt(tx_hash).await {
            Ok(Some(receipt)) => {
                let status = if receipt.status == Some(U64::from(1)) {
                    TransactionStatus::Confirmed
                } else {
                    TransactionStatus::Failed
                };
                debug!("Transaction status: {:?}", status);
                Ok(status)
            }
            Ok(None) => {
                // Transaction exists but not mined yet
                match self.provider.get_transaction(tx_hash).await {
                    Ok(Some(_)) => Ok(TransactionStatus::Pending),
                    Ok(None) => {
                        // If both receipt and transaction are not found, the transaction is unknown.
                        Ok(TransactionStatus::Unknown)
                    }
                    Err(e) => Err(anyhow::anyhow!(
                        "Failed to get transaction details for {}: {}",
                        tx_hash,
                        e
                    )),
                }
            }
            Err(e) => {
                warn!("Failed to get transaction receipt for {}: {}", tx_hash, e);
                Err(anyhow::anyhow!("Failed to get transaction receipt: {}", e))
            }
        }
    }

    async fn estimate_fee(&self, to_address: &str, amount: &str) -> Result<String> {
        debug!("Estimating fee for {} ETH to {}", amount, to_address);

        let _to_address = Address::from_str(to_address)
            .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;

        let _amount_wei =
            parse_ether(amount).map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

        let gas_price = self.get_gas_price().await?;
        let gas_limit = U256::from(21000u64); // Standard ETH transfer

        let total_fee = gas_price * gas_limit;
        let fee_eth = ethers::utils::format_ether(total_fee);

        debug!("Estimated fee: {} ETH", fee_eth);
        Ok(fee_eth)
    }

    async fn get_block_number(&self) -> Result<u64> {
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;

        Ok(block_number.as_u64())
    }

    fn validate_address(&self, address: &str) -> Result<bool> {
        match Address::from_str(address) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn get_network_name(&self) -> &str {
        &self.network_name
    }

    fn get_native_token(&self) -> &str {
        "ETH"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::providers::{Http, Provider};
    use std::convert::TryFrom;

    // helper to build a client without requiring a live RPC
    fn make_local_client() -> EthereumClient<Http> {
        let provider =
            Provider::<Http>::try_from("http://127.0.0.1:8545").expect("provider url ok");
        EthereumClient::new_with_provider(provider)
    }

    #[test]
    fn create_wallet_from_private_key_success() {
        let client = make_local_client();
        let key = [0x11u8; 32];
        let wallet = client.create_wallet_from_private_key(&key).expect("should create wallet");
        let _addr = wallet.address(); // basic smoke check
    }

    #[test]
    fn create_wallet_from_private_key_invalid_length() {
        let client = make_local_client();
        let short_key = [0u8; 16];
        let res = client.create_wallet_from_private_key(&short_key);
        assert!(res.is_err());
        let msg = format!("{}", res.unwrap_err());
        assert!(msg.contains("32") || msg.contains("Private key"), "unexpected err: {}", msg);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn send_transaction_short_key_fails_fast() {
        let client = make_local_client();
        let short_key = [0u8; 16];
        let res = client
            .send_transaction(&short_key, "0x0000000000000000000000000000000000000000", "0.1")
            .await;
        assert!(res.is_err());
    }

    #[test]
    fn test_address_validation_smoke() {
        let client = make_local_client();
        assert!(client.validate_address("0x742d35Cc6634C0532925a3b8D400e8B78fFe4860").unwrap());
        assert!(!client.validate_address("not-an-address").unwrap());
    }
}
// ...existing code...
