use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub quantum_safe: bool,
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub id: String,
    pub name: String,
    pub quantum_safe: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SendTransactionRequest {
    pub to_address: String,
    pub amount: String,
    pub network: String,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeAssetsRequest {
    pub from_wallet: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: String,
}

#[derive(Serialize)]
pub struct BridgeResponse {
    pub bridge_tx_id: String,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: String,
    pub network: String,
    pub symbol: String,
}

#[derive(Serialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<String>,
}

#[derive(Serialize)]
pub struct BackupResponse {
    pub seed_phrase: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RestoreWalletRequest {
    pub name: String,
    pub seed_phrase: String,
    #[serde(default)]
    pub quantum_safe: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MultiSigTransactionRequest {
    pub to_address: String,
    pub amount: String,
    pub network: String,
    pub signatures: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}
