// src/api/handlers.rs
use crate::api::server::ErrorResponse;
use crate::core::wallet_manager::WalletManager;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)] // 合并 Debug、Serialize 和 Deserialize
pub struct BridgeRequest {
    pub from_wallet: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub bridge_tx_id: String,
}

pub async fn bridge_assets(
    State(wallet_manager): State<Arc<WalletManager>>,
    Json(request): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Basic validation
    if request.from_wallet.is_empty()
        || request.from_chain.is_empty()
        || request.to_chain.is_empty()
        || request.token.is_empty()
        || request.amount.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid parameters".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    match wallet_manager.bridge_assets(
        &request.from_wallet,
        &request.from_chain,
        &request.to_chain,
        &request.token,
        &request.amount,
    ).await {
        Ok(bridge_tx_id) => Ok(Json(BridgeResponse { bridge_tx_id })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        )),
    }
}

pub async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn metrics_handler() -> &'static str {
    "# Prometheus metrics\n# TODO: Implement actual metrics"
}
