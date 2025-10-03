// src/api/server/handlers.rs
// 瀹屾暣鍐呭浠?src/api/bridge.rs 澶嶅埗

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use crate::core::wallet_manager::WalletManager;
use crate::api::server::AppState;

#[derive(Deserialize)]
pub struct BridgeRequest {
    pub from_wallet: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: String,
}

#[derive(serde::Serialize)]
pub struct BridgeResponse {
    pub bridge_tx_id: String,
}

pub async fn bridge_assets(
    State(state): State<AppState>,
    Json(request): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, StatusCode> {
    // 瀹炵幇妗ユ帴閫昏緫锛堣皟鐢?WalletManager::bridge_assets锛?    // 绠€鍖栫ず渚嬶細杩斿洖妯℃嫙鍝嶅簲
    let bridge_tx_id = format!("bridge-{}-{}", request.from_chain, request.to_chain);
    Ok(Json(BridgeResponse { bridge_tx_id }))
}
