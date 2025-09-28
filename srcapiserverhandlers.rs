// src/api/server/handlers.rs
// 完整内容从 src/api/bridge.rs 复制

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
pub struct BridgeResponse {
    pub bridge_tx_id: String,
}

pub async fn bridge_assets(
    State(state): State<AppState>,
    Json(request): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, StatusCode> {
    // 实现桥接逻辑（调用 WalletManager::bridge_assets）
    // 简化示例：返回模拟响应
    let bridge_tx_id = format!("bridge-{}-{}", request.from_chain, request.to_chain);
    Ok(Json(BridgeResponse { bridge_tx_id }))
}
