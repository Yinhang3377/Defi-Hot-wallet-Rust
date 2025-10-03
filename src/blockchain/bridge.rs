use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use rand::Rng; // 瀵煎叆 Rng trait 浠ヤ娇鐢?gen() 鏂规硶
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::blockchain::traits::BlockchainClient;
use crate::core::wallet_info::SecureWalletData;

/// Defines specific errors that can occur during a bridge operation.
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Unsupported chain pair: {0} to {1}")]
    UnsupportedChainPair(String, String),

    #[error("Insufficient liquidity for token {0}")]
    InsufficientLiquidity(String),

    #[error("Bridge contract error: {0}")]
    ContractError(String),

    #[error("Transaction timeout")]
    Timeout,
}
/// Represents the status of a cross-chain bridge transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeTransactionStatus {
    Initiated,
    SourceChainConfirmed,
    InTransit,
    DestinationChainPending,
    Completed,
    Failed(String),
}

/// Represents a cross-chain bridge transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: String,
    pub from_wallet: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: String,
    pub status: BridgeTransactionStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub fee_amount: Option<String>,
    pub estimated_completion_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// 瀹氫箟璺ㄩ摼妗ユ帴鍔熻兘鐨勯€氱敤 Trait
#[async_trait]
pub trait Bridge: Send + Sync {
    /// 鎵ц璺ㄩ摼璧勪骇杞Щ
    ///
    /// # Arguments
    /// * `from_chain` - 婧愰摼鍚嶇О (e.g., "eth")
    /// * `to_chain` - 鐩爣閾惧悕绉?(e.g., "solana")
    /// * `token` - 瑕佽浆绉荤殑浠ｅ竵绗﹀彿 (e.g., "USDC")
    /// * `amount` - 杞Щ鏁伴噺
    /// * `wallet_data` - 鍖呭惈瑙ｅ瘑鍚庝富瀵嗛挜鐨勫畨鍏ㄩ挶鍖呮暟鎹?    ///
    /// # Returns
    /// 杩斿洖婧愰摼涓婄殑浜ゆ槗鍝堝笇鎴栦竴涓敮涓€鐨勬ˉ鎺ユ搷浣淚D
    async fn transfer_across_chains(
        &self,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
        wallet_data: &SecureWalletData,
    ) -> Result<String>;

    /// 妫€鏌ユˉ鎺ヨ浆璐︾殑鐘舵€?    async fn check_transfer_status(&self, transfer_id: &str) -> Result<BridgeTransactionStatus>;
}

/// 涓€涓叿浣撶殑浠ュお鍧婂埌 Solana 鐨勬ˉ鎺ュ疄鐜帮紙妯℃嫙锛?pub struct EthereumToSolanaBridge {
    bridge_contract: String,
    eth_client: Option<Arc<dyn BlockchainClient>>,
    sol_client: Option<Arc<dyn BlockchainClient>>,
}
impl std::fmt::Debug for EthereumToSolanaBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumToSolanaBridge")
            .field("bridge_contract", &self.bridge_contract)
            .finish()
    }
}

impl EthereumToSolanaBridge {
    pub fn new(bridge_contract_address: &str) -> Self {
        Self {
            bridge_contract: bridge_contract_address.to_string(),
            eth_client: None,
            sol_client: None,
        }
    }

    // 娣诲姞楠岃瘉閫昏緫
    #[allow(dead_code)]
    async fn validate_bridge_params(
        &self,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
    ) -> Result<()> {
        // 楠岃瘉閾炬敮鎸?        if !["eth", "sepolia"].contains(&from_chain) {
            return Err(anyhow::anyhow!("Unsupported source chain: {}", from_chain));
        }

        if !["solana", "solana-devnet"].contains(&to_chain) {
            return Err(anyhow::anyhow!("Unsupported destination chain: {}", to_chain));
        }

        // 楠岃瘉浠ｅ竵鏀寔
        let supported_tokens = ["ETH", "USDC", "USDT", "DAI"];
        if !supported_tokens.contains(&token) {
            return Err(anyhow::anyhow!("Unsupported token: {}", token));
        }

        // 楠岃瘉閲戦
        let amount_float: f64 = amount.parse()?;
        if amount_float <= 0.0 {
            return Err(anyhow::anyhow!("Invalid amount: {}", amount));
        }

        Ok(())
    }

    #[allow(dead_code)]
    async fn check_liquidity(&self, to_chain: &str, token: &str, amount: &str) -> Result<bool> {
        // 鍦ㄥ疄闄呭疄鐜颁腑锛岃繖浼氭煡璇㈡ˉ鎺ュ悎绾︽垨娴佸姩鎬ф睜
        info!("[SIMULATED] Checking liquidity for {} {} on {}", amount, token, to_chain);

        // 妯℃嫙瀹炵幇锛岄殢鏈鸿繑鍥炴槸鍚︽湁瓒冲娴佸姩鎬?        let has_liquidity = rand::thread_rng().gen::<bool>();

        if !has_liquidity {
            info!("鈿狅笍 [SIMULATED] Insufficient liquidity for {} {} on {}", amount, token, to_chain);
        }

        Ok(has_liquidity)
    }

    pub fn with_clients(
        mut self,
        eth_client: Box<dyn BlockchainClient>,
        sol_client: Box<dyn BlockchainClient>,
    ) -> Result<Self> {
        // 楠岃瘉瀹㈡埛绔被鍨嬪吋瀹规€?        if !eth_client.get_network_name().contains("eth") {
            return Err(anyhow::anyhow!("Expected Ethereum client for source chain"));
        }

        if !sol_client.get_network_name().contains("solana") {
            return Err(anyhow::anyhow!("Expected Solana client for destination chain"));
        }

        self.eth_client = Some(Arc::from(eth_client));
        self.sol_client = Some(Arc::from(sol_client));
        Ok(self)
    }
}

#[async_trait]
impl Bridge for EthereumToSolanaBridge {
    async fn transfer_across_chains(
        &self,
        from_chain: &str,
        to_chain: &str,
        token: &str,
        amount: &str,
        _wallet_data: &SecureWalletData,
    ) -> Result<String> {
        info!(
            "馃寜 [SIMULATED] Initiating bridge transfer of {} {} from {} to {} via contract {}",
            amount, token, from_chain, to_chain, self.bridge_contract
        );

        // This is a mock implementation.
        let simulated_tx_hash = format!("0x_simulated_lock_tx_{}", Uuid::new_v4());
        info!("   - Source chain transaction hash: {}", simulated_tx_hash);
        info!("鉁?[SIMULATED] Bridge transfer initiated successfully.");

        Ok(simulated_tx_hash)
    }

    async fn check_transfer_status(&self, tx_hash: &str) -> Result<BridgeTransactionStatus> {
        info!("[SIMULATED] Checking status for transfer: {}", tx_hash);
        mock_check_transfer_status(tx_hash).await
    }
}

// 鐢ㄤ簬妯℃嫙鍜岃窡韪氦鏄撶姸鎬佺殑闈欐€佸瓨鍌?lazy_static! {
    static ref TRANSACTION_CHECKS: std::sync::Mutex<HashMap<String, u8>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 妯℃嫙妫€鏌ユˉ鎺ョ姸鎬佺殑杈呭姪鍑芥暟
async fn mock_check_transfer_status(tx_hash: &str) -> Result<BridgeTransactionStatus> {
    // 妯℃嫙缃戠粶寤惰繜
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 濡傛灉浜ゆ槗鍝堝笇鏄庣‘鍖呭惈"failed"锛岀洿鎺ヨ繑鍥炲け璐?    if tx_hash.contains("failed") {
        return Ok(BridgeTransactionStatus::Failed(
            "Transaction explicitly marked as failed".to_string(),
        ));
    }

    // Normalize tx key so tests can pre-seed counts using the base tx id
    // while still allowing markers appended to the tx_hash for forcing behavior.
    let normalized_key = if let Some(idx) = tx_hash.find("_force_ratio=") {
        &tx_hash[..idx]
    } else if let Some(idx) = tx_hash.find("_force_roll=") {
        &tx_hash[..idx]
    } else {
        tx_hash
    };

    // 鑾峰彇鎴栧垵濮嬪寲姝や氦鏄撶殑妫€鏌ユ鏁?(浣跨敤瑙勮寖鍖栭敭)
    let mut checks = TRANSACTION_CHECKS.lock().unwrap();
    let count = checks.entry(normalized_key.to_string()).or_insert(0);
    *count += 1;

    // 鍩轰簬妫€鏌ユ鏁板拰涓€浜涢殢鏈烘€у喅瀹氱姸鎬?    let current_count = *count;
    drop(checks); // 閲婃斁閿?
    // 闅忔満鏁扮敓鎴愬櫒
    let mut rng = rand::thread_rng();

    // Test-only deterministic hooks: allow tests to force a specific RNG outcome
    // by embedding markers in the tx_hash. Supported forms:
    //  - "force_ratio=true" or "force_ratio=false"  (used in the 1..=2 arm to emulate gen_ratio)
    //  - "force_roll=<n>" where <n> is an integer (used in later arms to emulate roll)
    // Use contains checks for ratio to avoid subtle parsing edge-cases and a robust
    // digit-parse for force_roll.
    let mut forced_ratio: Option<bool> = None;
    let mut forced_roll: Option<u32> = None;
    if tx_hash.contains("force_ratio=true") {
        forced_ratio = Some(true);
    } else if tx_hash.contains("force_ratio=false") {
        forced_ratio = Some(false);
    }
    if let Some(idx) = tx_hash.find("force_roll=") {
        let start = idx + "force_roll=".len();
        let tail = &tx_hash[start..];
        // parse consecutive digits robustly
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            if let Ok(v) = digits.parse::<u32>() {
                forced_roll = Some(v);
            }
        }
    }

    // 鐘舵€佽浆鎹㈤€昏緫:
    // 1-2娆℃鏌? 閫氬父鏄疘nTransit
    // 3-4娆℃鏌? 鍙兘瀹屾垚鎴栦粛鍦ㄨ繘琛?    // 5+娆℃鏌? 楂樻鐜囧畬鎴愶紝灏忔鐜囧け璐?    match current_count {
        1..=2 => {
            // 鍓嶄袱娆℃鏌ワ紝95%鏄疘nTransit
            if let Some(forced) = forced_ratio {
                if forced {
                    Ok(BridgeTransactionStatus::InTransit)
                } else {
                    Ok(BridgeTransactionStatus::Completed)
                }
            } else if rng.gen_ratio(95, 100) {
                Ok(BridgeTransactionStatus::InTransit)
            } else {
                // 5%鐨勫嚑鐜囧揩閫熷畬鎴愶紙骞歌繍鎯呭喌锛?                Ok(BridgeTransactionStatus::Completed)
            }
        }
        3..=4 => {
            // 绗?-4娆℃鏌ワ紝60%鏄疘nTransit锛?5%瀹屾垚锛?%澶辫触
            let roll: u32 = if let Some(v) = forced_roll { v } else { rng.gen_range(1..=100) };
            if roll <= 60 {
                Ok(BridgeTransactionStatus::InTransit)
            } else if roll <= 95 {
                Ok(BridgeTransactionStatus::Completed)
            } else {
                Ok(BridgeTransactionStatus::Failed("Network congestion detected".to_string()))
            }
        }
        _ => {
            // 绗?娆″強浠ヤ笂妫€鏌ワ紝20%鏄疘nTransit锛?0%瀹屾垚锛?0%澶辫触
            let roll: u32 = if let Some(v) = forced_roll { v } else { rng.gen_range(1..=100) };
            if roll <= 20 {
                Ok(BridgeTransactionStatus::InTransit)
            } else if roll <= 90 {
                Ok(BridgeTransactionStatus::Completed)
            } else {
                Ok(BridgeTransactionStatus::Failed("Slippage tolerance exceeded".to_string()))
            }
        }
    }
}

/// 妯℃嫙 Solana 鍒?Ethereum 鐨勬ˉ鎺?pub struct SolanaToEthereumBridge {
    bridge_contract: String,
}
impl std::fmt::Debug for SolanaToEthereumBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolanaToEthereumBridge")
            .field("bridge_contract", &self.bridge_contract)
            .finish()
    }
}
impl SolanaToEthereumBridge {
    pub fn new(addr: &str) -> Self {
        Self { bridge_contract: addr.to_string() }
    }
}
#[async_trait]
impl Bridge for SolanaToEthereumBridge {
    async fn transfer_across_chains(
        &self,
        from: &str,
        to: &str,
        tk: &str,
        amt: &str,
        _wd: &SecureWalletData,
    ) -> Result<String> {
        mock_bridge_transfer(from, to, tk, amt, &self.bridge_contract).await
    }
    async fn check_transfer_status(&self, tx_hash: &str) -> Result<BridgeTransactionStatus> {
        mock_check_transfer_status(tx_hash).await
    }
}

/// 妯℃嫙 Ethereum 鍒?BSC 鐨勬ˉ鎺?pub struct EthereumToBSCBridge {
    bridge_contract: String,
}
impl std::fmt::Debug for EthereumToBSCBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumToBSCBridge")
            .field("bridge_contract", &self.bridge_contract)
            .finish()
    }
}
impl EthereumToBSCBridge {
    pub fn new(addr: &str) -> Self {
        Self { bridge_contract: addr.to_string() }
    }
}
#[async_trait]
impl Bridge for EthereumToBSCBridge {
    async fn transfer_across_chains(
        &self,
        from: &str,
        to: &str,
        tk: &str,
        amt: &str,
        _wd: &SecureWalletData,
    ) -> Result<String> {
        mock_bridge_transfer(from, to, tk, amt, &self.bridge_contract).await
    }
    async fn check_transfer_status(&self, tx_hash: &str) -> Result<BridgeTransactionStatus> {
        mock_check_transfer_status(tx_hash).await
    }
}

/// 妯℃嫙 Polygon 鍒?Ethereum 鐨勬ˉ鎺?pub struct PolygonToEthereumBridge {
    bridge_contract: String,
}
impl std::fmt::Debug for PolygonToEthereumBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolygonToEthereumBridge")
            .field("bridge_contract", &self.bridge_contract)
            .finish()
    }
}
impl PolygonToEthereumBridge {
    pub fn new(addr: &str) -> Self {
        Self { bridge_contract: addr.to_string() }
    }
}
#[async_trait]
impl Bridge for PolygonToEthereumBridge {
    async fn transfer_across_chains(
        &self,
        from: &str,
        to: &str,
        tk: &str,
        amt: &str,
        _wd: &SecureWalletData,
    ) -> Result<String> {
        mock_bridge_transfer(from, to, tk, amt, &self.bridge_contract).await
    }
    async fn check_transfer_status(&self, tx_hash: &str) -> Result<BridgeTransactionStatus> {
        mock_check_transfer_status(tx_hash).await
    }
}

/// 妯℃嫙妗ユ帴璋冪敤鐨勮緟鍔╁嚱鏁?async fn mock_bridge_transfer(
    from: &str,
    to: &str,
    tk: &str,
    amt: &str,
    contract: &str,
) -> Result<String> {
    info!("馃寜 [SIMULATED] Bridge: {} {} from {} to {} via {}", amt, tk, from, to, contract);
    Ok(format!("0x_simulated_tx_{}", Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::traits::BlockchainClient;
    use crate::core::wallet_info::SecureWalletData;
    use anyhow::Result;
    use async_trait::async_trait;
    // Arc not required in tests here

    // A tiny mock client to test with_clients validation.
    struct MockClient {
        name: String,
    }

    #[async_trait]
    impl BlockchainClient for MockClient {
        fn clone_box(&self) -> Box<dyn BlockchainClient> {
            Box::new(MockClient { name: self.name.clone() })
        }

        async fn get_balance(&self, _address: &str) -> Result<String> {
            Ok("0".to_string())
        }

        async fn send_transaction(
            &self,
            _private_key: &[u8],
            _to_address: &str,
            _amount: &str,
        ) -> Result<String> {
            Ok("0xmocktx".to_string())
        }

        async fn get_transaction_status(
            &self,
            _tx_hash: &str,
        ) -> Result<crate::blockchain::traits::TransactionStatus> {
            Ok(crate::blockchain::traits::TransactionStatus::Confirmed)
        }

        async fn estimate_fee(&self, _to_address: &str, _amount: &str) -> Result<String> {
            Ok("0".to_string())
        }

        async fn get_block_number(&self) -> Result<u64> {
            Ok(0)
        }

        fn validate_address(&self, _address: &str) -> Result<bool> {
            Ok(true)
        }

        fn get_network_name(&self) -> &str {
            &self.name
        }

        fn get_native_token(&self) -> &str {
            "MOCK"
        }
    }

    fn make_wallet_data() -> SecureWalletData {
        // minimal SecureWalletData for passing into bridge methods
        SecureWalletData {
            info: crate::core::wallet_info::WalletInfo {
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

    #[tokio::test]
    async fn validate_bridge_params_rejects_bad_chains_tokens_and_amounts() {
        let b = EthereumToSolanaBridge::new("0xC");

        // unsupported source chain
        let res = b.validate_bridge_params("btc", "solana", "USDC", "1.0").await;
        assert!(res.is_err());
        assert!(format!("{}", res.unwrap_err()).contains("Unsupported source chain"));

        // unsupported destination chain
        let res2 = b.validate_bridge_params("eth", "bsc", "USDC", "1.0").await;
        assert!(res2.is_err());
        assert!(format!("{}", res2.unwrap_err()).contains("Unsupported destination chain"));

        // unsupported token
        let res3 = b.validate_bridge_params("eth", "solana", "FOO", "1.0").await;
        assert!(res3.is_err());
        assert!(format!("{}", res3.unwrap_err()).contains("Unsupported token"));

        // invalid amount (non-numeric)
        let res4 = b.validate_bridge_params("eth", "solana", "USDC", "abc").await;
        assert!(res4.is_err());

        // invalid amount (zero)
        let res5 = b.validate_bridge_params("eth", "solana", "USDC", "0").await;
        assert!(res5.is_err());
    }

    #[tokio::test]
    async fn check_liquidity_returns_bool_ok() {
        let b = EthereumToSolanaBridge::new("0xC");
        let r = b.check_liquidity("solana", "USDC", "1.0").await;
        assert!(r.is_ok());
        // can't reliably assert true/false because it's randomized; just verify type
        let _has = r.unwrap();
        let _ = _has;
    }

    #[tokio::test]
    async fn with_clients_validates_client_types_and_accepts_matching() {
        let eth = MockClient { name: "ethereum-mainnet".to_string() };
        let sol = MockClient { name: "solana-mainnet".to_string() };

        let bridge = EthereumToSolanaBridge::new("0xC");

        // correct types should succeed
        let res = bridge.with_clients(Box::new(eth), Box::new(sol));
        assert!(res.is_ok());

        // incorrect eth client (name doesn't contain "eth")
        let eth_bad = MockClient { name: "clientX".to_string() };
        let sol_ok = MockClient { name: "solana".to_string() };
        let res2 =
            EthereumToSolanaBridge::new("0xC").with_clients(Box::new(eth_bad), Box::new(sol_ok));
        assert!(res2.is_err());
        let err2 = res2.err().unwrap().to_string();
        assert!(err2.contains("Expected Ethereum client"));

        // incorrect sol client (name doesn't contain "solana")
        let eth_ok = MockClient { name: "ethclient".to_string() };
        let sol_bad = MockClient { name: "clientY".to_string() };
        let res3 =
            EthereumToSolanaBridge::new("0xC").with_clients(Box::new(eth_ok), Box::new(sol_bad));
        assert!(res3.is_err());
        let err3 = res3.err().unwrap().to_string();
        assert!(err3.contains("Expected Solana client"));
    }

    #[tokio::test]
    async fn transfer_across_chains_returns_simulated_hash_and_check_status_failed_marker(
    ) -> Result<()> {
        let bridge = EthereumToSolanaBridge::new("0xBridge");
        let w = make_wallet_data();

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
    async fn mock_bridge_transfer_variants_and_concurrent() -> Result<()> {
        let s2e = SolanaToEthereumBridge::new("0xS2E");
        let e2b = EthereumToBSCBridge::new("0xE2B");
        let poly = PolygonToEthereumBridge::new("0xP2E");
        let w = make_wallet_data();

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
                let w = make_wallet_data();
                async move { s2e.transfer_across_chains("solana", "eth", "USDC", "1.0", &w).await }
            }),
            tokio::spawn({
                let e2b = EthereumToBSCBridge::new("0xE2B");
                let w = make_wallet_data();
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

    #[tokio::test]
    async fn mock_check_transfer_status_respects_internal_counting() -> Result<()> {
        // ensure we can exercise different code paths by manipulating TRANSACTION_CHECKS
        let tx = "0x_test_counting";

        // set count to 0 (function will increment to 1) and call
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.remove(tx);
        }
        let s = mock_check_transfer_status(tx).await?;
        // first-time call should be InTransit or Completed 鈥?accept either
        assert!(matches!(
            s,
            BridgeTransactionStatus::InTransit | BridgeTransactionStatus::Completed
        ));

        // force failed by using explicit failed marker
        let sf = mock_check_transfer_status("this_failed_marker_failed").await?;
        assert!(matches!(sf, BridgeTransactionStatus::Failed(_)));

        Ok(())
    }

    #[tokio::test]
    async fn deterministic_mock_check_transfer_status_all_branches() -> Result<()> {
        // Clear any existing counts
        let tx = "0x_det_branch";
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.remove(tx);
        }

        // 1st check: force_ratio=false to deterministically return Completed on 1..=2 arm
        let tx1 = format!("{}_force_ratio=false", tx);
        let s1 = mock_check_transfer_status(&tx1).await?;
        assert_eq!(s1, BridgeTransactionStatus::Completed);

        // Reset count and test 1..=2 InTransit via force_ratio=true
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.remove(tx);
        }
        let tx2 = format!("{}_force_ratio=true", tx);
        let s2 = mock_check_transfer_status(&tx2).await?;
        assert_eq!(s2, BridgeTransactionStatus::InTransit);

        // For 3..=4 arm, we need current_count to be 3; pre-seed counts accordingly
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 2u8); // next call increments to 3
        }
        // force roll to 50 -> InTransit (<=60)
        let tx3 = format!("{}_force_roll=50", tx);
        let s3 = mock_check_transfer_status(&tx3).await?;
        assert_eq!(s3, BridgeTransactionStatus::InTransit);

        // seed back to 2 -> next call 3 and force roll 80 -> Completed (<=95)
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 2u8);
        }
        let tx4 = format!("{}_force_roll=80", tx);
        let s4 = mock_check_transfer_status(&tx4).await?;
        assert_eq!(s4, BridgeTransactionStatus::Completed);

        // seed back to 2 -> next call 3 and force roll 99 -> Failed (>
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 2u8);
        }
        let tx5 = format!("{}_force_roll=99", tx);
        let s5 = mock_check_transfer_status(&tx5).await?;
        assert!(matches!(s5, BridgeTransactionStatus::Failed(_)));

        // For 5+ arm, seed count to 5
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 5u8); // next call increments to 6
        }
        // force roll 10 -> InTransit (<=20)
        let tx6 = format!("{}_force_roll=10", tx);
        let s6 = mock_check_transfer_status(&tx6).await?;
        assert_eq!(s6, BridgeTransactionStatus::InTransit);

        // seed to 5 and force roll 50 -> Completed (<=90)
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 5u8);
        }
        let tx7 = format!("{}_force_roll=50", tx);
        let s7 = mock_check_transfer_status(&tx7).await?;
        assert_eq!(s7, BridgeTransactionStatus::Completed);

        // seed to 5 and force roll 95 -> Failed (>90)
        {
            let mut m = TRANSACTION_CHECKS.lock().unwrap();
            m.insert(tx.to_string(), 5u8);
        }
        let tx8 = format!("{}_force_roll=95", tx);
        let s8 = mock_check_transfer_status(&tx8).await?;
        assert!(matches!(s8, BridgeTransactionStatus::Failed(_)));

        Ok(())
    }
}
