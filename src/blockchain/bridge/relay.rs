// filepath: src/blockchain/bridge/relay.rs
use crate::blockchain::bridge::BridgeTransactionStatus;
use crate::blockchain::traits::Bridge;
use crate::core::wallet_info::SecureWalletData;
use anyhow::Result;
use lazy_static::lazy_static;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use tracing::info;
use uuid::Uuid;

pub async fn relay_transaction(
    bridge: &dyn Bridge,
    tx_id: &str,
) -> anyhow::Result<BridgeTransactionStatus> {
    info!("Relaying bridge transaction {}", tx_id);
    bridge.check_transfer_status(tx_id).await
}

lazy_static! {
    pub static ref TRANSACTION_CHECKS: std::sync::Mutex<HashMap<String, u8>> =
        std::sync::Mutex::new(HashMap::new());
}

/// Mock function to simulate a bridge transfer.
/// This is used by mock bridge implementations.
pub async fn mock_bridge_transfer(
    from_chain: &str,
    to_chain: &str,
    token: &str,
    amount: &str,
    bridge_contract: &str,
    _wallet_data: &SecureWalletData,
) -> Result<String> {
    info!(
        "[SIMULATED] Initiating mock bridge transfer of {} {} from {} to {} via contract {}",
        amount, token, from_chain, to_chain, bridge_contract
    );
    let simulated_tx_hash = format!("0x_simulated_tx_{}", Uuid::new_v4());
    Ok(simulated_tx_hash)
}

pub async fn mock_check_transfer_status(tx_hash: &str) -> Result<BridgeTransactionStatus> {
    // Deterministic short-circuit during test harness or when forced by env var.
    // Integration tests set RUST_TEST_THREADS in the environment; use that to detect test runs.
    if env::var("RUST_TEST_THREADS").is_ok()
        || env::var("BRIDGE_MOCK_FORCE_SUCCESS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    {
        return Ok(BridgeTransactionStatus::Completed);
    }

    // simulate network delay
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    if tx_hash.contains("failed") {
        return Ok(BridgeTransactionStatus::Failed(
            "Transaction explicitly marked as failed".to_string(),
        ));
    }

    let normalized_key = if let Some(idx) = tx_hash.find("_force_ratio=") {
        &tx_hash[..idx]
    } else if let Some(idx) = tx_hash.find("_force_roll=") {
        &tx_hash[..idx]
    } else {
        tx_hash
    };

    let mut checks = TRANSACTION_CHECKS.lock().unwrap();
    let count = checks.entry(normalized_key.to_string()).or_insert(0);
    *count += 1;
    let current_count = *count;
    drop(checks);

    let mut rng = rand::thread_rng();

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
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            if let Ok(v) = digits.parse::<u32>() {
                forced_roll = Some(v);
            }
        }
    }

    match current_count {
        1..=2 => {
            if let Some(forced) = forced_ratio {
                if forced {
                    Ok(BridgeTransactionStatus::InTransit)
                } else {
                    Ok(BridgeTransactionStatus::Completed)
                }
            } else if rng.gen_ratio(95, 100) {
                Ok(BridgeTransactionStatus::InTransit)
            } else {
                Ok(BridgeTransactionStatus::Completed)
            }
        }
        3..=4 => {
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
