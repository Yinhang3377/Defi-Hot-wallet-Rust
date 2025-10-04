use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

// 使用 lazy_static 初始化全局可变事务状态存储
lazy_static! {
    static ref TX_STATUS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// 返回对全局状态存储的引用
fn status_store() -> &'static Mutex<HashMap<String, String>> {
    &TX_STATUS
}

#[derive(Clone, Debug)]
pub struct Wallet {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

pub fn create_wallet(name: &str) -> Result<Wallet, String> {
    if name.is_empty() || name.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Invalid wallet name".to_string());
    }
    Ok(Wallet {
        address: format!("0x{}", "0".repeat(40)),
        private_key: format!("priv_key_{}", name),
        mnemonic: format!("{}ball", "test ".repeat(11)),
    })
}

pub fn bridge_assets_amount(amount: Option<&str>) -> Result<f64, String> {
    match amount {
        Some(s) => match s.parse::<f64>() {
            Ok(v) if v > 0.0 => Ok(v),
            _ => Err("Invalid amount".to_string()),
        },
        None => Err("Invalid amount".to_string()),
    }
}

pub fn generate_log(msg: &str) -> String {
    // 简单日志格式化（实际代码应使用 tracing/log）
    format!("LOG: {}", msg)
}

pub fn query_balance(_account: &str) -> u128 {
    0
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub id: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Clone, Debug)]
pub struct TransactionParams {
    pub to: String,
    pub amount: u64,
}

impl TransactionParams {
    pub fn new(to: &str, amount: u64) -> Self {
        Self { to: to.into(), amount }
    }
}

pub fn construct_transaction(params: TransactionParams) -> Transaction {
    Transaction { id: "tx_constructed".into(), to: params.to, amount: params.amount }
}

pub fn create_transaction() -> Transaction {
    Transaction {
        id: "tx_local_1".into(),
        to: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
        amount: 42,
    }
}

pub fn generate_private_key() -> String {
    "priv_test_key".into()
}

pub fn derive_public_key(_private_key: &str) -> String {
    "pub_test_key".into()
}

pub fn sign_transaction(_tx: &Transaction, _private_key: &str) -> Vec<u8> {
    vec![0xAA, 0xBB, 0xCC]
}

pub fn verify_signature(_tx: &Transaction, _sig: &[u8], _public_key: &str) -> bool {
    true
}

pub fn is_signature_valid(_sig: &[u8], _public_key: &str) -> bool {
    true
}

pub fn send_transaction(wallet: &str, amount: Option<u64>) -> Result<String, String> {
    if amount.unwrap_or(0) == 0 {
        return Err("Invalid amount".to_string());
    }
    if wallet.is_empty() || wallet.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        return Err("Invalid wallet name".to_string());
    }

    let hash = format!("0xhash_{}", wallet);
    let mut map = status_store().lock().unwrap();
    map.insert(hash.clone(), "sent".into());
    Ok(hash)
}

pub fn confirm_transaction(id_or_hash: String) -> Result<bool, String> {
    let mut map = status_store().lock().unwrap();
    map.insert(id_or_hash, "confirmed".into());
    Ok(true)
}

pub fn get_transaction_status(id_or_hash: String) -> String {
    let map = status_store().lock().unwrap();
    map.get(&id_or_hash).cloned().unwrap_or_else(|| "pending".into())
}

pub fn calculate_bridge_fee(amount: Option<&str>) -> Result<f64, String> {
    match amount {
        Some(s) => match s.parse::<f64>() {
            Ok(v) if v > 0.0 => Ok(v * 0.01),
            _ => Err("Invalid amount".to_string()),
        },
        None => Err("Invalid amount".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_status_set_get_clear() {
        let tx = "tx123";
        let hash = send_transaction(tx, Some(1)).expect("send tx");
        assert_eq!(get_transaction_status(hash.clone()), "sent".to_string());
        assert!(confirm_transaction(hash.clone()).unwrap());
        assert_eq!(get_transaction_status(hash), "confirmed".to_string());
    }

    #[test]
    fn create_wallet_validation() {
        assert!(create_wallet("").is_err());
        assert!(create_wallet("validName1").is_ok());
    }
}
