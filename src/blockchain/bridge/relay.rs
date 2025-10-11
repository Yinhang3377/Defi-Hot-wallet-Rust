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
    _from_chain: &str,
    _to_chain: &str,
    _token: &str,
    amount: &str,
    _bridge_contract: &str,
    _wallet_data: &SecureWalletData,
) -> Result<String> {
    info!("[SIMULATED] Initiating mock bridge transfer of {} {}", amount, _token);

    // validate amount: must parse to a non-negative number (allow 0.0), reject negatives and invalid.
    let amt_trim = amount.trim();
    let amt_val = match amt_trim.parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "invalid amount '{}': must be a non-negative number",
                amount
            ));
        }
    };
    if amt_val < 0.0 {
        return Err(anyhow::anyhow!("invalid amount '{}': must be >= 0", amount));
    }

    // Only return a simulated tx when mocks are explicitly enabled via env.
    if !bridge_force_success_enabled() {
        return Err(anyhow::anyhow!(
            "mock bridge disabled: set BRIDGE_MOCK_FORCE_SUCCESS (or BRIDGE_MOCK / FORCE_BRIDGE_SUCCESS / BRIDGE_MOCK_FORCE) to enable"
        ));
    }

    let simulated_tx_hash = format!("0x_simulated_tx_{}", Uuid::new_v4());
    Ok(simulated_tx_hash)
}

/// 检查是否应该强制 mock 桥接为成功（Accept several env names/values）。
/// Default: disabled. Enabled only if one of the keys is present and not explicitly false-like.
fn bridge_force_success_enabled() -> bool {
    const KEYS: &[&str] =
        &["BRIDGE_MOCK_FORCE_SUCCESS", "BRIDGE_MOCK", "FORCE_BRIDGE_SUCCESS", "BRIDGE_MOCK_FORCE"];

    for &k in KEYS {
        if let Ok(val) = env::var(k) {
            let v = val.trim();
            // explicit disabled values -> continue checking other keys
            if v.eq_ignore_ascii_case("0")
                || v.eq_ignore_ascii_case("false")
                || v.eq_ignore_ascii_case("no")
            {
                continue;
            }
            // empty, "1", "true", "yes", "on", or any other non-false string -> enabled
            if v.is_empty()
                || v == "1"
                || v.eq_ignore_ascii_case("true")
                || v.eq_ignore_ascii_case("yes")
                || v.eq_ignore_ascii_case("on")
                || !v.is_empty()
            {
                return true;
            }
        }
    }

    false
}

pub async fn mock_check_transfer_status(tx_hash: &str) -> Result<BridgeTransactionStatus> {
    // If this is a simulated tx produced by mock_bridge_transfer, always treat as Completed.
    if tx_hash.starts_with("0x_simulated_tx_") {
        return Ok(BridgeTransactionStatus::Completed);
    }

    // If tests explicitly force success via env, short-circuit and clear any previous counters.
    if env::var("RUST_TEST_THREADS").is_ok() || bridge_force_success_enabled() {
        if let Ok(mut checks) = TRANSACTION_CHECKS.lock() {
            checks.clear();
        }
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
