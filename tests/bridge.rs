// src/api/handlers/bridge.rs
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use defi_hot_wallet::api::server::ErrorResponse;
use defi_hot_wallet::core::wallet_manager::WalletManager;

#[derive(Debug, Deserialize)]
pub struct BridgeRequest {
    pub from_wallet: String,
    pub from_chain: String,
    pub to_chain: String,
    pub token: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub bridge_tx_id: String,
}

pub async fn bridge_assets(
    State(wallet_manager): State<Arc<WalletManager>>,
    Json(request): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "🌉 API call to bridge assets for wallet '{}': {} {} from {} to {}",
        request.from_wallet, request.amount, request.token, request.from_chain, request.to_chain
    );

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
        Err(e) => {
            warn!("Failed to bridge assets: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e.to_string(), code: "BRIDGE_FAILED".to_string() }),
            ))
        }
    }
}
