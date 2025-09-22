use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static TX_STATUS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
fn status_store() -> &'static Mutex<HashMap<String, String>> {
    TX_STATUS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Debug)]
pub struct Wallet {
    pub address: String,
    pub private_key: String,
    pub mnemonic: String,
}

pub fn create_wallet(_password: &str) -> Wallet {
    Wallet {
        address: "0x0000000000000000000000000000000000000000".into(),
        private_key: "priv_test_key".into(),
        mnemonic: "test test test test test test test test test test test ball".into(),
    }
}

pub fn generate_log(msg: &str) -> String {
    // 真实实现可接入 tracing/log
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
        Self {
            to: to.into(),
            amount,
        }
    }
}

pub fn construct_transaction(params: TransactionParams) -> Transaction {
    Transaction {
        id: "tx_constructed".into(),
        to: params.to,
        amount: params.amount,
    }
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

pub fn send_transaction(tx: Transaction) -> Result<String, String> {
    // 伪造 TxHash，并置状态 sent
    let hash = format!("0xhash_{}", tx.id);
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
    map.get(&id_or_hash)
        .cloned()
        .unwrap_or_else(|| "pending".into())
}
