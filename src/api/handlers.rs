// src/api/handlers.rs
use crate::api::types::{BridgeAssetsRequest, BridgeResponse, ErrorResponse};
use crate::core::wallet_manager::WalletManager;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::sync::Arc;

/// Business logic for bridge assets endpoint.
/// Accepts a State-wrapped Arc<WalletManager> so callers (server layer)
/// can perform authentication before delegating to this function.
pub async fn bridge_assets(
    State(wallet_manager): State<Arc<WalletManager>>,
    Json(request): Json<BridgeAssetsRequest>,
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

    if request.amount.parse::<f64>().unwrap_or(-1.0) <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid amount".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    // 修复：将不支持的链错误统一为 404 NOT_FOUND
    if request.from_chain != "eth" && request.from_chain != "solana" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Unsupported chain".to_string(),
                code: "BRIDGE_FAILED".to_string(),
            }),
        ));
    }

    match wallet_manager
        .bridge_assets(
            &request.from_wallet,
            &request.from_chain,
            &request.to_chain,
            &request.token,
            &request.amount,
        )
        .await
    {
        Ok(bridge_tx_id) => Ok(Json(BridgeResponse { bridge_tx_id })),
        Err(err) => {
            // 在返回 500 错误前，记录详细的底层错误信息和请求内容
            // 直接打印到 stderr，确保在测试输出里能看到底层错误（临时调试）
            eprintln!("DEBUG_BRIDGE_ERROR: {:?}", err);
            tracing::error!(error = %err, request = ?request, "bridge_assets handler failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to bridge assets".to_string(),
                    code: "BRIDGE_FAILED".to_string(),
                }),
            ))
        }
    }
}

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn metrics_handler() -> String {
    "# HELP defi_hot_wallet_requests_total Total number of requests\n# TYPE defi_hot_wallet_requests_total counter\ndefi_hot_wallet_requests_total 0\n".to_string()
}
