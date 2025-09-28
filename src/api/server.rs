use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::core::config::WalletConfig;
use crate::core::errors::WalletError;
use crate::core::wallet_manager::WalletManager;

#[derive(Clone)]
pub struct WalletServer {
    pub wallet_manager: Arc<WalletManager>,
    pub config: WalletConfig,
    pub api_key: Option<String>,
}

impl WalletServer {
    pub async fn new(
        _host: String,
        _port: u16,
        config: WalletConfig,
        api_key: Option<String>,
    ) -> Result<Self, WalletError> {
        let wallet_manager = Arc::new(WalletManager::new(&config).await?);
        Ok(Self { wallet_manager, config, api_key })
    }

    pub async fn create_router(self) -> Router {
        let state = Arc::new(self);
        Router::new()
            .route("/api/health", get(health_check))
            .route("/api/wallets", post(create_wallet).get(list_wallets))
            .route("/api/wallets/:name", delete(delete_wallet))
            .route("/api/wallets/:name/balance", get(get_balance))
            .route("/api/wallets/:name/send", post(send_transaction))
            .route("/api/wallets/:name/history", get(get_transaction_history))
            .route("/api/wallets/:name/backup", get(backup_wallet))
            .route("/api/wallets/restore", post(restore_wallet))
            .route("/api/wallets/:name/send_multi_sig", post(send_multi_sig_transaction))
            .route("/api/bridge", post(bridge_assets))
            .route("/api/metrics", get(metrics))
            .layer(
                ServiceBuilder::new()
                    .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB 请求体限制（速率限制）
                    .layer(TraceLayer::new_for_http()),
            ) // 日志
            .with_state(state)
    }

    pub async fn run(self, host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router().await;
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Server running on {}", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn authenticate(headers: &HeaderMap, api_key: &Option<String>) -> Result<(), StatusCode> {
    if let Some(key) = api_key {
        if let Some(auth) = headers.get("Authorization") {
            if auth == key {
                return Ok(());
            }
        }
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize)]
pub struct RestoreWalletRequest {
    pub name: String,
    pub seed_phrase: String,
}

#[derive(Deserialize)]
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

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),  // 补丁：添加版本
        "timestamp": chrono::Utc::now().to_rfc3339()  // 补丁：添加时间戳
    }))
}

async fn create_wallet(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<WalletResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if payload.name.is_empty() || payload.name.contains(|c: char| !c.is_alphanumeric() && c != '_')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid wallet name".to_string(),
                code: "WALLET_CREATION_FAILED".to_string(),
            }),
        ));
    }

    match state.wallet_manager.create_wallet(&payload.name, payload.quantum_safe).await {
        Ok(_) => Ok(Json(WalletResponse {
            id: payload.name.clone(),
            name: payload.name,
            quantum_safe: payload.quantum_safe,
        })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create wallet".to_string(),
                code: "WALLET_CREATION_FAILED".to_string(),
            }),
        )),
    }
}

async fn list_wallets(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
) -> Result<Json<Vec<WalletResponse>>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            let response = wallets
                .into_iter()
                .map(|w| WalletResponse {
                    id: w.name.clone(),
                    name: w.name,
                    quantum_safe: w.quantum_safe,
                })
                .collect();
            Ok(Json(response))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to list wallets".to_string(),
                code: "LIST_WALLETS_FAILED".to_string(),
            }),
        )),
    }
}

async fn delete_wallet(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if name.is_empty() || name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid wallet name".to_string(),
                code: "DELETE_WALLET_FAILED".to_string(),
            }),
        ));
    }

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == name) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "DELETE_WALLET_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "DELETE_WALLET_FAILED".to_string(),
                }),
            ))
        }
    }

    match state.wallet_manager.delete_wallet(&name).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to delete wallet".to_string(),
                code: "DELETE_WALLET_FAILED".to_string(),
            }),
        )),
    }
}

#[derive(Deserialize)]
pub struct BalanceQuery {
    pub network: String,
}

async fn get_balance(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(query): Query<BalanceQuery>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if name.is_empty() || query.network.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid parameters".to_string(),
                code: "GET_BALANCE_FAILED".to_string(),
            }),
        ));
    }

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == name) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "GET_BALANCE_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "GET_BALANCE_FAILED".to_string(),
                }),
            ))
        }
    }

    match state.wallet_manager.get_balance(&name, &query.network).await {
        Ok(balance) => {
            let symbol = match query.network.as_str() {
                "eth" => "ETH",
                "solana" => "SOL",
                _ => "UNKNOWN",
            };
            Ok(Json(BalanceResponse {
                balance,
                network: query.network,
                symbol: symbol.to_string(),
            }))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to get balance".to_string(),
                code: "GET_BALANCE_FAILED".to_string(),
            }),
        )),
    }
}

async fn send_transaction(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<SendTransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if name.is_empty()
        || payload.to_address.is_empty()
        || payload.amount.is_empty()
        || payload.network.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid parameters".to_string(),
                code: "TRANSACTION_FAILED".to_string(),
            }),
        ));
    }

    if payload.network == "eth" && !payload.to_address.starts_with("0x") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid address format".to_string(),
                code: "TRANSACTION_FAILED".to_string(),
            }),
        ));
    }

    if payload.amount.parse::<f64>().unwrap_or(-1.0) <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid amount".to_string(),
                code: "TRANSACTION_FAILED".to_string(),
            }),
        ));
    }

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == name) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "TRANSACTION_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "TRANSACTION_FAILED".to_string(),
                }),
            ))
        }
    }

    match state
        .wallet_manager
        .send_transaction(&name, &payload.to_address, &payload.amount, &payload.network)
        .await
    {
        Ok(tx_hash) => Ok(Json(TransactionResponse { tx_hash, status: "sent".to_string() })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to send transaction".to_string(),
                code: "TRANSACTION_FAILED".to_string(),
            }),
        )),
    }
}

async fn get_transaction_history(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<TransactionHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == name) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "HISTORY_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "HISTORY_FAILED".to_string(),
                }),
            ))
        }
    }

    match state.wallet_manager.get_transaction_history(&name).await {
        Ok(history) => Ok(Json(TransactionHistoryResponse { transactions: history })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to get history".to_string(),
                code: "HISTORY_FAILED".to_string(),
            }),
        )),
    }
}

async fn backup_wallet(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<BackupResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == name) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "BACKUP_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "BACKUP_FAILED".to_string(),
                }),
            ))
        }
    }

    match state.wallet_manager.backup_wallet(&name).await {
        Ok(seed) => Ok(Json(BackupResponse { seed_phrase: seed })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to backup".to_string(),
                code: "BACKUP_FAILED".to_string(),
            }),
        )),
    }
}

async fn restore_wallet(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Json(payload): Json<RestoreWalletRequest>,
) -> Result<Json<WalletResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    match state.wallet_manager.restore_wallet(&payload.name, &payload.seed_phrase).await {
        Ok(_) => Ok(Json(WalletResponse {
            id: payload.name.clone(),
            name: payload.name,
            quantum_safe: false,
        })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to restore".to_string(),
                code: "RESTORE_FAILED".to_string(),
            }),
        )),
    }
}

async fn send_multi_sig_transaction(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<MultiSigTransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if payload.signatures.len() < state.config.multi_sig_threshold as usize {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Insufficient signatures".to_string(),
                code: "MULTI_SIG_FAILED".to_string(),
            }),
        ));
    }

    match state
        .wallet_manager
        .send_multi_sig_transaction(
            &name,
            &payload.to_address,
            &payload.amount,
            &payload.network,
            &payload.signatures,
        )
        .await
    {
        Ok(tx_hash) => Ok(Json(TransactionResponse { tx_hash, status: "sent".to_string() })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to send multi-sig transaction".to_string(),
                code: "MULTI_SIG_FAILED".to_string(),
            }),
        )),
    }
}

async fn bridge_assets(
    State(state): State<Arc<WalletServer>>,
    headers: HeaderMap,
    Json(payload): Json<BridgeAssetsRequest>,
) -> Result<Json<BridgeResponse>, (StatusCode, Json<ErrorResponse>)> {
    authenticate(&headers, &state.api_key).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
                code: "AUTH_FAILED".to_string(),
            }),
        )
    })?;

    if payload.from_wallet.is_empty()
        || payload.from_chain.is_empty()
        || payload.to_chain.is_empty()
        || payload.token.is_empty()
        || payload.amount.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid parameters".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    if payload.amount.parse::<f64>().unwrap_or(-1.0) <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid amount".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    if payload.from_chain != "eth" && payload.from_chain != "solana" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Unsupported chain".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    match state.wallet_manager.list_wallets().await {
        Ok(wallets) => {
            if !wallets.iter().any(|w| w.name == payload.from_wallet) {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Wallet not found".to_string(),
                        code: "BRIDGE_FAILED".to_string(),
                    }),
                ));
            }
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check wallet".to_string(),
                    code: "BRIDGE_FAILED".to_string(),
                }),
            ))
        }
    }

    match state
        .wallet_manager
        .bridge_assets(
            &payload.from_wallet,
            &payload.from_chain,
            &payload.to_chain,
            &payload.token,
            &payload.amount,
        )
        .await
    {
        Ok(bridge_tx_id) => Ok(Json(BridgeResponse { bridge_tx_id })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to bridge assets".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        )),
    }
}

async fn metrics() -> String {
    "# HELP defi_hot_wallet_requests_total Total number of requests\n# TYPE defi_hot_wallet_requests_total counter\ndefi_hot_wallet_requests_total 0\n".to_string()
}
