use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

// 浣跨敤 lazy_static 杩涜绾跨▼瀹夊叏鐨勫崟娆″垵濮嬪寲
lazy_static! {
    static ref TX_STATUS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// 杈呭姪鍑芥暟锛岀敤浜庤幏鍙栧叏灞€鐨勭姸鎬佸瓨鍌?fn status_store() -> &'static Mutex<HashMap<String, String>> {
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
        address: "0x".to_string() + &"0".repeat(40),
        private_key: "priv_key_".to_string() + name,
        mnemonic: "test ".repeat(11) + "ball",
    })
}

pub fn bridge_assets_amount(amount: Option<&str>) -> Result<f64, String> {
    match amount {
        Some(s) if s.parse::<f64>().is_ok() && s.parse::<f64>().unwrap() > 0.0 => {
            Ok(s.parse().unwrap())
        }
        _ => Err("Invalid amount".to_string()),
    }
}

pub fn generate_log(msg: &str) -> String {
    // 鐪熷疄瀹炵幇鍙帴鍏?tracing/log
    format!("LOG: {msg}")
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
    if amount.is_none() || amount.unwrap() == 0 {
        // 楠岃瘉閲戦
        return Err("Invalid amount".to_string());
    }
    if wallet.is_empty() || wallet.chars().any(|c| !c.is_alphanumeric() && c != '_') {
        // 楠岃瘉閽卞寘鍚嶇О
        return Err("Invalid wallet name".to_string());
    }
    // 鐢熸垚涓€涓ā鎷熺殑浜ゆ槗鍝堝笇
    let hash = format!("0xhash_{}", wallet);
    // 鑾峰彇鐘舵€佸瓨鍌ㄧ殑閿侊紝骞舵彃鍏ユ柊浜ゆ槗鐨勭姸鎬佷负 "sent"
    let mut map = status_store().lock().unwrap();
    map.insert(hash.clone(), "sent".into());
    Ok(hash)
}

pub fn confirm_transaction(id_or_hash: String) -> Result<bool, String> {
    // 鑾峰彇鐘舵€佸瓨鍌ㄧ殑閿侊紝骞舵洿鏂颁氦鏄撶姸鎬佷负 "confirmed"
    let mut map = status_store().lock().unwrap();
    map.insert(id_or_hash, "confirmed".into());
    Ok(true)
}

pub fn get_transaction_status(id_or_hash: String) -> String {
    // 鑾峰彇鐘舵€佸瓨鍌ㄧ殑閿侊紝骞舵煡璇氦鏄撶姸鎬?    let map = status_store().lock().unwrap();
    map.get(&id_or_hash).cloned().unwrap_or_else(|| "pending".into()) // 濡傛灉鎵句笉鍒帮紝鍒欒繑鍥?"pending"
}

pub fn calculate_bridge_fee(amount: Option<&str>) -> Result<f64, String> {
    match amount {
        Some(s) if s.parse::<f64>().is_ok() && s.parse::<f64>().unwrap() > 0.0 => {
            Ok(s.parse::<f64>().unwrap() * 0.01)
        }
        _ => Err("Invalid amount".to_string()),
    }
}
